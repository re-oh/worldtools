use std::mem::size_of;

use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};
use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::{
    geo::GeoPoint,
    seed::WorldSeed,
    tile::{TILE_APRON, TILE_STORAGE_SAMPLES, TileId, storage_index},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainSettings {
    pub planet_radius_m: f64,
    pub sea_level_m: f32,
    pub continental_frequency: f32,
    pub continental_bias: f32,
    pub ocean_depth_m: f32,
    pub lowland_relief_m: f32,
    pub mountain_relief_m: f32,
    pub detail_relief_m: f32,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        Self {
            planet_radius_m: 6_371_000.0,
            sea_level_m: 0.0,
            continental_frequency: 0.82,
            continental_bias: -0.04,
            ocean_depth_m: 5_500.0,
            lowland_relief_m: 2_700.0,
            mountain_relief_m: 4_600.0,
            detail_relief_m: 180.0,
        }
    }
}

impl TerrainSettings {
    /// Stable fingerprint for cache keys and persisted tile metadata.
    #[must_use]
    pub fn fingerprint(self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new_derive_key("worldtools.terrain-settings.v1");
        hasher.update(&self.planet_radius_m.to_bits().to_le_bytes());
        hasher.update(&self.sea_level_m.to_bits().to_le_bytes());
        hasher.update(&self.continental_frequency.to_bits().to_le_bytes());
        hasher.update(&self.continental_bias.to_bits().to_le_bytes());
        hasher.update(&self.ocean_depth_m.to_bits().to_le_bytes());
        hasher.update(&self.lowland_relief_m.to_bits().to_le_bytes());
        hasher.update(&self.mountain_relief_m.to_bits().to_le_bytes());
        hasher.update(&self.detail_relief_m.to_bits().to_le_bytes());
        *hasher.finalize().as_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TerrainTileStats {
    pub minimum_m: f32,
    pub maximum_m: f32,
    pub mean_m: f32,
    pub standard_deviation_m: f32,
    pub land_fraction: f32,
}

#[derive(Debug, Clone)]
pub struct TerrainTile {
    pub id: TileId,
    pub stats: TerrainTileStats,
    elevation_m: Box<[f32]>,
}

impl TerrainTile {
    #[must_use]
    pub fn elevation_m(&self) -> &[f32] {
        &self.elevation_m
    }

    /// # Panics
    /// Panics when either storage coordinate is outside `0..259`.
    #[must_use]
    pub fn storage_sample(&self, storage_x: usize, storage_y: usize) -> f32 {
        assert!(storage_x < TILE_STORAGE_SAMPLES && storage_y < TILE_STORAGE_SAMPLES);
        self.elevation_m[storage_index(storage_x, storage_y)]
    }

    #[must_use]
    pub fn interior_sample(&self, sample_x: usize, sample_y: usize) -> f32 {
        self.storage_sample(sample_x + TILE_APRON, sample_y + TILE_APRON)
    }

    #[must_use]
    pub fn byte_len(&self) -> usize {
        size_of::<Self>() + self.elevation_m.len() * size_of::<f32>()
    }
}

/// A continuous sphere-space height source. It intentionally produces only a
/// conservative base surface; tectonics, erosion, and hydrology can refine it
/// as separate tile stages without changing the geometry contract.
pub struct TerrainGenerator {
    seed: WorldSeed,
    settings: TerrainSettings,
    continental: FastNoiseLite,
    mountain: FastNoiseLite,
    detail: FastNoiseLite,
}

impl TerrainGenerator {
    #[must_use]
    pub fn new(seed: WorldSeed, settings: TerrainSettings) -> Self {
        let continental = configured_noise(
            seed.key("terrain.continental").noise_seed(),
            settings.continental_frequency,
            FractalType::FBm,
            5,
            0.52,
        );
        let mountain = configured_noise(
            seed.key("terrain.mountain").noise_seed(),
            1.9,
            FractalType::Ridged,
            5,
            0.48,
        );
        let detail = configured_noise(
            seed.key("terrain.detail").noise_seed(),
            7.5,
            FractalType::FBm,
            4,
            0.45,
        );
        Self {
            seed,
            settings,
            continental,
            mountain,
            detail,
        }
    }

    #[must_use]
    pub const fn seed(&self) -> WorldSeed {
        self.seed
    }

    #[must_use]
    pub const fn settings(&self) -> TerrainSettings {
        self.settings
    }

    #[must_use]
    pub fn sample_elevation_m(&self, direction: DVec3) -> f32 {
        let direction = direction.try_normalize().unwrap_or(DVec3::X);
        let continental =
            Self::sample(&self.continental, direction) + self.settings.continental_bias;
        let signed_land = continental - 0.015;

        if signed_land <= 0.0 {
            let abyss = (-signed_land / 0.75).clamp(0.0, 1.0).powf(0.72);
            return self.settings.sea_level_m - self.settings.ocean_depth_m * abyss;
        }

        let inland = smoothstep(0.0, 0.42, signed_land);
        let lowlands = signed_land.powf(0.82) * self.settings.lowland_relief_m;
        let ridge = ((Self::sample(&self.mountain, direction) + 1.0) * 0.5).clamp(0.0, 1.0);
        let mountain_gate = smoothstep(0.18, 0.72, inland) * smoothstep(0.48, 0.9, ridge);
        let mountains = mountain_gate * self.settings.mountain_relief_m;
        let detail = Self::sample(&self.detail, direction) * self.settings.detail_relief_m * inland;
        self.settings.sea_level_m + lowlands + mountains + detail
    }

    #[must_use]
    pub fn sample_geo(&self, point: GeoPoint) -> f32 {
        self.sample_elevation_m(point.direction())
    }

    #[must_use]
    pub fn generate(&self, id: TileId) -> TerrainTile {
        let mut elevation_m = vec![0.0_f32; TILE_STORAGE_SAMPLES * TILE_STORAGE_SAMPLES];
        for storage_y in 0..TILE_STORAGE_SAMPLES {
            for storage_x in 0..TILE_STORAGE_SAMPLES {
                let direction = id.storage_sample_direction(storage_x, storage_y);
                elevation_m[storage_index(storage_x, storage_y)] =
                    self.sample_elevation_m(direction);
            }
        }

        let stats = statistics(&elevation_m, self.settings.sea_level_m);
        TerrainTile {
            id,
            stats,
            elevation_m: elevation_m.into_boxed_slice(),
        }
    }

    fn sample(noise: &FastNoiseLite, direction: DVec3) -> f32 {
        noise.get_noise_3d(direction.x, direction.y, direction.z)
    }
}

fn configured_noise(
    seed: i32,
    frequency: f32,
    fractal: FractalType,
    octaves: i32,
    gain: f32,
) -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(seed);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2S));
    noise.set_frequency(Some(frequency));
    noise.set_fractal_type(Some(fractal));
    noise.set_fractal_octaves(Some(octaves));
    noise.set_fractal_lacunarity(Some(2.0));
    noise.set_fractal_gain(Some(gain));
    noise
}

fn smoothstep(edge_0: f32, edge_1: f32, value: f32) -> f32 {
    let t = ((value - edge_0) / (edge_1 - edge_0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[allow(clippy::cast_possible_truncation)] // Summaries match the f32 height storage format.
fn statistics(samples: &[f32], sea_level_m: f32) -> TerrainTileStats {
    let mut minimum = f32::INFINITY;
    let mut maximum = f32::NEG_INFINITY;
    let mut sum = 0.0_f64;
    let mut land_samples = 0_usize;

    for &sample in samples {
        minimum = minimum.min(sample);
        maximum = maximum.max(sample);
        sum += f64::from(sample);
        land_samples += usize::from(sample > sea_level_m);
    }

    let sample_count =
        f64::from(u32::try_from(samples.len()).expect("a terrain tile sample count fits in u32"));
    let mean = sum / sample_count;
    let variance = samples
        .iter()
        .map(|&sample| {
            let delta = f64::from(sample) - mean;
            delta * delta
        })
        .sum::<f64>()
        / sample_count;
    let land_sample_count = f64::from(
        u32::try_from(land_samples).expect("a terrain tile land sample count fits in u32"),
    );

    TerrainTileStats {
        minimum_m: minimum,
        maximum_m: maximum,
        mean_m: mean as f32,
        standard_deviation_m: variance.sqrt() as f32,
        land_fraction: (land_sample_count / sample_count) as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::{CubeFace, TILE_CELLS};

    #[test]
    fn generation_is_deterministic() {
        let id = TileId::new(CubeFace::PositiveY, 3, 2, 5).unwrap();
        let generator = TerrainGenerator::new(WorldSeed(91), TerrainSettings::default());
        assert_eq!(
            generator.generate(id).elevation_m(),
            generator.generate(id).elevation_m()
        );
    }

    #[test]
    fn adjacent_tiles_have_identical_shared_heights() {
        let generator = TerrainGenerator::new(WorldSeed(12), TerrainSettings::default());
        let left = generator.generate(TileId::new(CubeFace::PositiveZ, 4, 6, 9).unwrap());
        let right = generator.generate(TileId::new(CubeFace::PositiveZ, 4, 7, 9).unwrap());
        for y in 0..=TILE_CELLS {
            assert_eq!(
                left.interior_sample(TILE_CELLS, y).to_bits(),
                right.interior_sample(0, y).to_bits()
            );
        }
    }

    #[test]
    fn parent_and_child_shared_samples_have_identical_heights() {
        let generator = TerrainGenerator::new(WorldSeed(0x5151), TerrainSettings::default());
        let parent_id = TileId::new(CubeFace::NegativeX, 4, 3, 11).unwrap();
        let child_id = parent_id.children().unwrap()[3];
        let parent = generator.generate(parent_id);
        let child = generator.generate(child_id);

        for child_y in (0..=TILE_CELLS).step_by(2) {
            for child_x in (0..=TILE_CELLS).step_by(2) {
                assert_eq!(
                    child.interior_sample(child_x, child_y).to_bits(),
                    parent
                        .interior_sample(TILE_CELLS / 2 + child_x / 2, TILE_CELLS / 2 + child_y / 2)
                        .to_bits(),
                );
            }
        }
    }
}
