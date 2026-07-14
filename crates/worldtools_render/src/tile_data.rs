use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    prelude::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use worldtools_world::{EditJournal, GeoPoint, TerrainGenerator, TerrainSettings, WorldSeed};

use crate::projection::{
    MAP_TILE_APRON, MAP_TILE_CELLS, MAP_TILE_SAMPLE_COUNT, MAP_TILE_SAMPLES, MapTileId,
};

#[derive(Clone, Debug)]
pub struct MapTileData {
    pub id: MapTileId,
    pub range_m: [f32; 2],
    samples: Arc<[f32]>,
}

impl MapTileData {
    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)] // Level 17 bounds grid indices well inside i64 and exact f64 integers.
    pub fn generate(id: MapTileId, seed: WorldSeed, settings: TerrainSettings) -> Self {
        Self::generate_inner(id, seed, settings, None)
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)] // Level 17 bounds grid indices well inside i64 and exact f64 integers.
    pub fn generate_with_edits(
        id: MapTileId,
        seed: WorldSeed,
        settings: TerrainSettings,
        edits: &EditJournal,
    ) -> Self {
        Self::generate_inner(id, seed, settings, Some(edits))
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)] // Level 17 bounds grid indices well inside i64 and exact f64 integers.
    fn generate_inner(
        id: MapTileId,
        seed: WorldSeed,
        settings: TerrainSettings,
        edits: Option<&EditJournal>,
    ) -> Self {
        let generator = TerrainGenerator::new(seed, settings);
        let x_cells = u64::from(id.x_extent()) * u64::from(MAP_TILE_CELLS);
        let y_cells = u64::from(id.y_extent()) * u64::from(MAP_TILE_CELLS);
        let mut samples = Vec::with_capacity(MAP_TILE_SAMPLE_COUNT);
        let mut range_m = [f32::INFINITY, f32::NEG_INFINITY];

        for storage_y in 0..MAP_TILE_SAMPLES {
            let local_y = i64::from(storage_y) - i64::from(MAP_TILE_APRON);
            let global_y =
                (i64::from(id.y) * i64::from(MAP_TILE_CELLS) + local_y).clamp(0, y_cells as i64);
            let latitude = 90.0 - 180.0 * global_y as f64 / y_cells as f64;

            for storage_x in 0..MAP_TILE_SAMPLES {
                let local_x = i64::from(storage_x) - i64::from(MAP_TILE_APRON);
                let global_x = i64::from(id.x) * i64::from(MAP_TILE_CELLS) + local_x;
                let longitude = -180.0 + 360.0 * global_x as f64 / x_cells as f64;
                let point = GeoPoint::from_degrees(latitude, longitude);
                let base = generator.sample_geo(point);
                let elevation = edits.map_or(base, |journal| {
                    journal.apply_elevation(point.direction(), base, settings.planet_radius_m)
                });
                range_m[0] = range_m[0].min(elevation);
                range_m[1] = range_m[1].max(elevation);
                samples.push(elevation);
            }
        }

        Self {
            id,
            range_m,
            samples: samples.into(),
        }
    }

    #[must_use]
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    #[must_use]
    pub fn byte_len(&self) -> usize {
        size_of::<Self>() + self.samples.len() * size_of::<f32>()
    }

    #[must_use]
    pub(crate) fn to_image(&self) -> Image {
        Image::new(
            Extent3d {
                width: MAP_TILE_SAMPLES,
                height: MAP_TILE_SAMPLES,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bytemuck::cast_slice(self.samples()).to_vec(),
            TextureFormat::R32Float,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(tile: &MapTileData, x: u32, y: u32) -> f32 {
        let index = usize::try_from(y * MAP_TILE_SAMPLES + x).unwrap();
        tile.samples()[index]
    }

    #[test]
    fn neighboring_map_tiles_share_exact_boundaries() {
        let seed = WorldSeed(9);
        let settings = TerrainSettings::default();
        let left = MapTileData::generate(
            MapTileId {
                level: 2,
                x: 3,
                y: 1,
            },
            seed,
            settings,
        );
        let right = MapTileData::generate(
            MapTileId {
                level: 2,
                x: 4,
                y: 1,
            },
            seed,
            settings,
        );
        for y in 0..MAP_TILE_SAMPLES {
            assert_eq!(
                sample(&left, MAP_TILE_APRON + MAP_TILE_CELLS, y).to_bits(),
                sample(&right, MAP_TILE_APRON, y).to_bits()
            );
        }
    }
}
