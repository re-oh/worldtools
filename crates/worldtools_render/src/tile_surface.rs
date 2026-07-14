use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bevy::{
    prelude::*,
    sprite_render::{Material2dPlugin, MeshMaterial2d},
    window::PrimaryWindow,
};

use crate::{
    blue_noise,
    debug::{RenderDebugSettings, TileRenderStats},
    display::{MapDisplayMode, MapDisplaySettings},
    projection::{MAP_TILE_APRON, MAP_TILE_CELLS, MapTileId, MapTilePlacement},
    streaming::{MapTileStreamer, VisibleMapTiles},
    tile_data::MapTileData,
    tile_material::{TerrainTileMaterial, TerrainTileMaterialParams},
    view::{MapView, MapViewport},
};
use worldtools_simulation::WorldDataLayer;

const PLANET_RADIUS_M: f32 = 6_371_000.0;

#[derive(Resource)]
struct TileRenderAssets {
    mesh: Handle<Mesh>,
    blue_noise: Handle<Image>,
}

#[derive(Debug)]
struct GpuTile {
    data: Arc<MapTileData>,
    elevation: Handle<Image>,
    layer: Handle<Image>,
}

#[derive(Default, Resource)]
struct GpuTileCache(HashMap<MapTileId, GpuTile>);

impl GpuTileCache {
    fn images_for(
        &mut self,
        data: Arc<MapTileData>,
        images: &mut Assets<Image>,
    ) -> (Handle<Image>, Handle<Image>) {
        use std::collections::hash_map::Entry;

        match self.0.entry(data.id) {
            Entry::Occupied(mut entry) => {
                if !Arc::ptr_eq(&entry.get().data, &data) {
                    let elevation = images.add(data.elevation_image());
                    let layer = images.add(data.layer_image());
                    entry.insert(GpuTile {
                        data,
                        elevation,
                        layer,
                    });
                }
                (entry.get().elevation.clone(), entry.get().layer.clone())
            }
            Entry::Vacant(entry) => {
                let elevation = images.add(data.elevation_image());
                let layer = images.add(data.layer_image());
                entry.insert(GpuTile {
                    data,
                    elevation: elevation.clone(),
                    layer: layer.clone(),
                });
                (elevation, layer)
            }
        }
    }
}

#[derive(Debug)]
struct RenderedTile {
    entity: Entity,
    material: Handle<TerrainTileMaterial>,
    source: MapTileId,
    layer: WorldDataLayer,
}

#[derive(Default, Resource)]
struct RenderedTiles(HashMap<MapTilePlacement, RenderedTile>);

pub(crate) struct TileSurfacePlugin;

impl Plugin for TileSurfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuTileCache>()
            .init_resource::<RenderedTiles>()
            .init_resource::<TileRenderStats>()
            .add_plugins(Material2dPlugin::<TerrainTileMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(Update, sync_surfaces);
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.insert_resource(TileRenderAssets {
        mesh: meshes.add(Rectangle::default()),
        blue_noise: images.add(blue_noise::image()),
    });
}

#[allow(
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines
)] // Bevy systems expose independent resources as value wrapper parameters.
fn sync_surfaces(
    mut commands: Commands,
    visible: Res<VisibleMapTiles>,
    streamer: Res<MapTileStreamer>,
    debug: Res<RenderDebugSettings>,
    display: Res<MapDisplaySettings>,
    view: Res<MapView>,
    viewport: Res<MapViewport>,
    shared: Res<TileRenderAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainTileMaterial>>,
    mut transforms: Query<&mut Transform>,
    mut gpu_tiles: ResMut<GpuTileCache>,
    mut rendered: ResMut<RenderedTiles>,
    mut stats: ResMut<TileRenderStats>,
    mut rendered_epoch: Local<u64>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let fallback = Vec2::new(window.width(), window.height());
    let viewport_size = viewport.size(fallback);
    let viewport_min = if viewport.max.x > viewport.min.x {
        viewport.min
    } else {
        Vec2::ZERO
    };
    let mut active = HashSet::with_capacity(visible.0.placements.len());
    let mut next_stats = TileRenderStats::default();

    if *rendered_epoch != streamer.world_epoch() {
        for (_, tile) in rendered.0.drain() {
            commands.entity(tile.entity).despawn();
            materials.remove(&tile.material);
        }
        for (_, tile) in gpu_tiles.0.drain() {
            images.remove(&tile.elevation);
            images.remove(&tile.layer);
        }
        *rendered_epoch = streamer.world_epoch();
    }

    for &placement in &visible.0.placements {
        active.insert(placement);
        let transform = placement_transform(
            placement,
            *view,
            viewport_min,
            viewport_size,
            Vec2::new(window.width(), window.height()),
        );
        let source = if let Some(exact) = streamer.get(placement.id) {
            next_stats.exact += 1;
            exact
        } else if let Some(tile) = rendered.0.get(&placement) {
            next_stats.stale += 1;
            if let Some(mut material) = materials.get_mut(&tile.material) {
                material.params = tile_params(
                    placement.id,
                    tile.source,
                    stale_display(*display, tile.layer, streamer.active_layer()),
                    *debug,
                    true,
                );
            }
            if let Ok(mut current) = transforms.get_mut(tile.entity) {
                *current = transform;
            }
            continue;
        } else if let Some(fallback) = streamer.best_available(placement.id) {
            next_stats.fallback += 1;
            fallback
        } else {
            next_stats.missing += 1;
            continue;
        };
        let (elevation, layer) = gpu_tiles.images_for(source.clone(), &mut images);
        let params = tile_params(placement.id, source.id, *display, *debug, false);
        if let Some(tile) = rendered.0.get_mut(&placement) {
            if let Some(mut material) = materials.get_mut(&tile.material) {
                material.elevation = elevation;
                material.layer = layer;
                material.params = params;
            }
            tile.source = source.id;
            tile.layer = source.layer;
            if let Ok(mut current) = transforms.get_mut(tile.entity) {
                *current = transform;
            }
        } else {
            let material = materials.add(TerrainTileMaterial {
                params,
                elevation,
                blue_noise: shared.blue_noise.clone(),
                layer,
            });
            let entity = commands
                .spawn((
                    Mesh2d(shared.mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    transform,
                    Name::new(format!(
                        "Map tile L{} {}/{}",
                        placement.id.level, placement.id.x, placement.id.y
                    )),
                ))
                .id();
            rendered.0.insert(
                placement,
                RenderedTile {
                    entity,
                    material,
                    source: source.id,
                    layer: source.layer,
                },
            );
        }
    }

    rendered.0.retain(|placement, tile| {
        if active.contains(placement) {
            true
        } else {
            commands.entity(tile.entity).despawn();
            materials.remove(&tile.material);
            false
        }
    });

    gpu_tiles.0.retain(|id, _| streamer.contains(*id));
    next_stats.rendered = rendered.0.len();
    next_stats.gpu_resident = gpu_tiles.0.len();
    if debug.trace_streaming && *stats != next_stats {
        tracing::debug!(
            target: "worldtools_render::surface",
            rendered = next_stats.rendered,
            exact = next_stats.exact,
            fallback = next_stats.fallback,
            stale = next_stats.stale,
            missing = next_stats.missing,
            gpu_resident = next_stats.gpu_resident,
            "tile surface state changed"
        );
    }
    *stats = next_stats;
}

fn stale_display(
    display: MapDisplaySettings,
    resident_layer: WorldDataLayer,
    active_layer: WorldDataLayer,
) -> MapDisplaySettings {
    if resident_layer == active_layer {
        display
    } else {
        MapDisplaySettings {
            mode: MapDisplayMode::Physical,
            ..display
        }
    }
}

#[allow(clippy::cast_precision_loss)] // Level 17 caps tile coordinates below exact f32 integers.
fn tile_params(
    desired: MapTileId,
    source: MapTileId,
    display: MapDisplaySettings,
    debug: RenderDebugSettings,
    stale: bool,
) -> TerrainTileMaterialParams {
    debug_assert!(source.level <= desired.level);
    let scale = 1_u32 << (desired.level - source.level);
    let relative_x = desired.x - source.x * scale;
    let relative_y = desired.y - source.y * scale;
    let source_span = MAP_TILE_CELLS as f32 / scale as f32;
    let origin_x = MAP_TILE_APRON as f32 + relative_x as f32 * source_span;
    let origin_y = MAP_TILE_APRON as f32 + relative_y as f32 * source_span;

    let tiles_y = (1_u32 << source.level) as f32;
    let metres_per_sample =
        std::f32::consts::PI * PLANET_RADIUS_M / (tiles_y * MAP_TILE_CELLS as f32);
    let latitude = std::f32::consts::FRAC_PI_2
        - std::f32::consts::PI * (desired.y as f32 + 0.5) / desired.y_extent() as f32;

    TerrainTileMaterialParams {
        sample_rect: Vec4::new(origin_x, origin_y, source_span, source_span),
        metrics: Vec4::new(
            (metres_per_sample * latitude.cos().abs()).max(0.01),
            metres_per_sample,
            0.7,
            0.0,
        ),
        debug: Vec4::new(
            debug.shader_flags(stale) as f32,
            debug.border_width_px.clamp(0.5, 8.0),
            f32::from(desired.level),
            f32::from(source.level),
        ),
        display: Vec4::from_array(display.shader_values()),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)] // Convert only camera-relative values to render-space f32.
fn placement_transform(
    placement: MapTilePlacement,
    view: MapView,
    viewport_min: Vec2,
    viewport_size: Vec2,
    window_size: Vec2,
) -> Transform {
    let x_extent = placement.id.x_extent() as f32;
    let y_extent = placement.id.y_extent() as f32;
    let aspect = viewport_size.x / viewport_size.y.max(1.0);
    let horizontal_span = view.horizontal_span(aspect);
    let relative = Vec2::new(
        (((placement.unwrapped_x as f64 + 0.5) / f64::from(x_extent)) - view.center.x) as f32,
        (((f64::from(placement.id.y) + 0.5) / f64::from(y_extent)) - view.center.y) as f32,
    );
    let screen = viewport_min
        + Vec2::new(
            (0.5 + relative.x / horizontal_span) * viewport_size.x,
            (0.5 + relative.y / view.vertical_span) * viewport_size.y,
        );
    let size = Vec2::new(
        viewport_size.x / (horizontal_span * x_extent),
        viewport_size.y / (view.vertical_span * y_extent),
    );

    Transform {
        translation: Vec3::new(
            screen.x - window_size.x * 0.5,
            window_size.y * 0.5 - screen.y,
            1.0,
        ),
        scale: size.extend(1.0),
        ..default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worldtools_world::{TerrainSettings, WorldSeed};

    #[test]
    fn regenerated_tile_replaces_its_gpu_image() {
        let id = MapTileId {
            level: 0,
            x: 0,
            y: 0,
        };
        let first = Arc::new(MapTileData::generate(
            id,
            WorldSeed(1),
            TerrainSettings::default(),
        ));
        let replacement = Arc::new(MapTileData::generate(
            id,
            WorldSeed(2),
            TerrainSettings::default(),
        ));
        let mut images = Assets::<Image>::default();
        let mut cache = GpuTileCache::default();

        let first_image = cache.images_for(first.clone(), &mut images);
        let reused_image = cache.images_for(first, &mut images);
        let replacement_image = cache.images_for(replacement, &mut images);

        assert_eq!(first_image.0.id(), reused_image.0.id());
        assert_eq!(first_image.1.id(), reused_image.1.id());
        assert_ne!(first_image.0.id(), replacement_image.0.id());
        assert_ne!(first_image.1.id(), replacement_image.1.id());
    }

    #[test]
    fn descendant_fallback_selects_its_parent_quadrant() {
        let source = MapTileId {
            level: 2,
            x: 3,
            y: 1,
        };
        let desired = MapTileId {
            level: 4,
            x: 14,
            y: 6,
        };
        let params = tile_params(
            desired,
            source,
            MapDisplaySettings::default(),
            RenderDebugSettings::default(),
            false,
        );
        assert!((params.sample_rect.z - 64.0).abs() < f32::EPSILON);
        assert!((params.sample_rect.w - 64.0).abs() < f32::EPSILON);
        assert!((params.sample_rect.x - 129.0).abs() < f32::EPSILON);
        assert!((params.sample_rect.y - 129.0).abs() < f32::EPSILON);
    }

    #[test]
    fn debug_params_describe_fallback_and_stale_state() {
        let source = MapTileId {
            level: 2,
            x: 1,
            y: 1,
        };
        let desired = MapTileId {
            level: 3,
            x: 2,
            y: 2,
        };
        let debug = RenderDebugSettings {
            tile_borders: true,
            lod_tint: true,
            residency_tint: true,
            border_width_px: 3.0,
            trace_streaming: false,
            freeze_streaming: false,
        };

        let params = tile_params(desired, source, MapDisplaySettings::default(), debug, true);
        assert_eq!(params.debug, Vec4::new(15.0, 3.0, 3.0, 2.0));
    }

    #[test]
    fn display_changes_only_uniform_presentation_values() {
        let desired = MapTileId {
            level: 4,
            x: 14,
            y: 6,
        };
        let source = MapTileId {
            level: 2,
            x: 3,
            y: 1,
        };
        let physical = tile_params(
            desired,
            source,
            MapDisplaySettings::default(),
            RenderDebugSettings::default(),
            false,
        );
        let contours = tile_params(
            desired,
            source,
            MapDisplaySettings {
                mode: crate::MapDisplayMode::Contours,
                contour_interval_m: 100.0,
                ..MapDisplaySettings::default()
            },
            RenderDebugSettings::default(),
            false,
        );

        assert_eq!(physical.sample_rect, contours.sample_rect);
        assert_eq!(physical.metrics, contours.metrics);
        assert_eq!(physical.debug, contours.debug);
        assert_ne!(physical.display, contours.display);
        assert_eq!(contours.display, Vec4::new(4.0, 0.0, 100.0, 1.0));
    }

    #[test]
    fn stale_overlay_data_uses_physical_elevation_until_replaced() {
        let climate = MapDisplaySettings {
            mode: MapDisplayMode::Climate,
            ..MapDisplaySettings::default()
        };

        assert_eq!(
            stale_display(climate, WorldDataLayer::Tectonics, WorldDataLayer::Climate).mode,
            MapDisplayMode::Physical
        );
        assert_eq!(
            stale_display(climate, WorldDataLayer::Climate, WorldDataLayer::Climate).mode,
            MapDisplayMode::Climate
        );
    }

    #[test]
    fn tiles_are_square_for_two_to_one_projection() {
        let placement = MapTilePlacement {
            id: MapTileId {
                level: 3,
                x: 8,
                y: 4,
            },
            unwrapped_x: 8,
        };
        let transform = placement_transform(
            placement,
            MapView::default(),
            Vec2::ZERO,
            Vec2::new(1600.0, 800.0),
            Vec2::new(1600.0, 800.0),
        );
        assert!((transform.scale.x - transform.scale.y).abs() < 0.001);
    }
}
