use bevy::{prelude::*, sprite_render::Material2dPlugin, window::PrimaryWindow};

use crate::{
    blue_noise,
    debug::RenderDebugSettings,
    height_field::{HeightField, HeightFieldUpload},
    material::{TerrainMaterial, TerrainMaterialParams},
    streaming::TileStreamingPlugin,
    tile_surface::TileSurfacePlugin,
    view::{self, MapView, MapViewport},
};

#[derive(Debug, Resource)]
pub struct TerrainSurface {
    pub entity: Entity,
    pub material: Handle<TerrainMaterial>,
    pub elevation: Handle<Image>,
}

pub struct WorldToolsRenderPlugin;

impl Plugin for WorldToolsRenderPlugin {
    fn build(&self, app: &mut App) {
        crate::material::register_shader(app);
        crate::tile_material::register_shader(app);
        app.init_resource::<MapView>()
            .init_resource::<MapViewport>()
            .init_resource::<RenderDebugSettings>()
            .add_message::<HeightFieldUpload>()
            .add_plugins((
                Material2dPlugin::<TerrainMaterial>::default(),
                TileStreamingPlugin,
                TileSurfacePlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    view::navigate,
                    resize_surface,
                    update_material,
                    upload_height_field,
                ),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let field = HeightField::flat(4, 2, -100.0);
    let range = field.range_m();
    let elevation = images.add(field.to_image());
    let blue_noise = images.add(blue_noise::image());
    let window_size = windows.single().map_or(Vec2::new(1280.0, 720.0), |window| {
        Vec2::new(window.width(), window.height())
    });

    let mut params = TerrainMaterialParams::default();
    params.display.x = window_size.x;
    params.display.y = window_size.y;
    params.style.x = range[0];
    params.style.y = range[1];
    let material = materials.add(TerrainMaterial {
        params,
        elevation: elevation.clone(),
        blue_noise,
    });

    commands.spawn((
        Camera2d,
        Camera {
            order: -100,
            ..default()
        },
    ));
    let entity = commands
        .spawn((
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(material.clone()),
            Transform::from_scale(window_size.extend(1.0)),
            Name::new("WorldTools terrain surface"),
        ))
        .id();

    commands.insert_resource(TerrainSurface {
        entity,
        material,
        elevation,
    });
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn resize_surface(
    surface: Option<Res<TerrainSurface>>,
    viewport: Res<MapViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut transforms: Query<&mut Transform>,
) {
    let (Some(surface), Ok(window)) = (surface, windows.single()) else {
        return;
    };
    let Ok(mut transform) = transforms.get_mut(surface.entity) else {
        return;
    };
    let window_size = Vec2::new(window.width(), window.height());
    let size = viewport.size(window_size);
    let min = if viewport.max.x > viewport.min.x {
        viewport.min
    } else {
        Vec2::ZERO
    };
    let center = min + size * 0.5;
    transform.translation.x = center.x - window_size.x * 0.5;
    transform.translation.y = window_size.y * 0.5 - center.y;
    transform.scale = size.extend(1.0);
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn update_material(
    surface: Option<Res<TerrainSurface>>,
    view: Res<MapView>,
    viewport: Res<MapViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let (Some(surface), Ok(window)) = (surface, windows.single()) else {
        return;
    };
    let Some(mut material) = materials.get_mut(&surface.material) else {
        return;
    };
    let fallback = Vec2::new(window.width(), window.height());
    let size = viewport.size(fallback);
    let aspect = size.x / size.y.max(1.0);
    material.params.view = Vec4::new(
        view.center.x,
        view.center.y,
        view.horizontal_span(aspect),
        view.vertical_span,
    );
    material.params.display.x = size.x;
    material.params.display.y = size.y;
}

fn upload_height_field(
    surface: Option<Res<TerrainSurface>>,
    mut uploads: MessageReader<HeightFieldUpload>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let Some(surface) = surface else {
        return;
    };
    for upload in uploads.read() {
        let Some(mut image) = images.get_mut(&surface.elevation) else {
            continue;
        };
        *image = upload.0.to_image();
        if let Some(mut material) = materials.get_mut(&surface.material) {
            let range = upload.0.range_m();
            material.params.style.x = range[0];
            material.params.style.y = range[1];
        }
    }
}
