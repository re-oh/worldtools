use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TerrainAudit;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainAggregate {
    pub tile_count: usize,
    pub sea_level_m: f32,
    pub elevation_vertex_samples: usize,
    pub non_finite_elevation_vertex_samples: usize,
    pub derivative_samples: usize,
    pub elevation_minimum_m: f32,
    pub elevation_maximum_m: f32,
    pub elevation_vertex_weighted_mean_m: f64,
    pub elevation_vertex_weighted_standard_deviation_m: f64,
    pub land_vertex_weighted_fraction: f64,
    pub slope_gradient_root_mean_square: f64,
    pub slope_degrees_root_mean_square: f64,
    pub ruggedness_root_mean_square_m: f64,
    pub local_extrema_fraction: f64,
}

#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum TerrainAggregateError {
    #[error("cannot aggregate terrain audits with sea levels {expected_m} m and {found_m} m")]
    MixedSeaLevels { expected_m: f32, found_m: f32 },
}

/// Combines per-tile audits using sample-count weighting and stable moments.
///
/// # Errors
/// Returns [`TerrainAggregateError::MixedSeaLevels`] when the inputs do not
/// use one common sea level.
pub fn aggregate_terrain<'a>(
    audits: impl IntoIterator<Item = &'a TerrainAudit>,
) -> Result<Option<TerrainAggregate>, TerrainAggregateError> {
    let mut aggregate = AggregateState::default();
    for audit in audits {
        aggregate.add(audit)?;
    }
    Ok(aggregate.finish())
}

#[derive(Default)]
struct AggregateState {
    tile_count: usize,
    sea_level_m: Option<f32>,
    elevation_samples: usize,
    non_finite_samples: usize,
    mean: f64,
    elevation_m2: f64,
    minimum: f32,
    maximum: f32,
    land_samples: usize,
    derivative_samples: usize,
    slope_gradient_squared: f64,
    slope_degrees_squared: f64,
    ruggedness_squared: f64,
    extrema_samples: usize,
}

impl AggregateState {
    fn add(&mut self, audit: &TerrainAudit) -> Result<(), TerrainAggregateError> {
        if let Some(expected_m) = self.sea_level_m {
            if expected_m.to_bits() != audit.sea_level_m.to_bits() {
                return Err(TerrainAggregateError::MixedSeaLevels {
                    expected_m,
                    found_m: audit.sea_level_m,
                });
            }
        } else {
            self.sea_level_m = Some(audit.sea_level_m);
        }
        let distribution = &audit.elevation;
        let incoming_count = distribution.finite_sample_count;
        if incoming_count > 0 {
            if self.elevation_samples == 0 {
                self.minimum = distribution.minimum;
                self.maximum = distribution.maximum;
                self.mean = distribution.mean;
                self.elevation_m2 =
                    distribution.standard_deviation.powi(2) * count_as_f64(incoming_count);
            } else {
                self.minimum = self.minimum.min(distribution.minimum);
                self.maximum = self.maximum.max(distribution.maximum);
                let existing_count = count_as_f64(self.elevation_samples);
                let incoming_count_f64 = count_as_f64(incoming_count);
                let combined_count = existing_count + incoming_count_f64;
                let delta = distribution.mean - self.mean;
                self.elevation_m2 += distribution.standard_deviation.powi(2) * incoming_count_f64
                    + delta * delta * existing_count * incoming_count_f64 / combined_count;
                self.mean += delta * incoming_count_f64 / combined_count;
            }
            self.elevation_samples += incoming_count;
        }

        self.tile_count += 1;
        self.non_finite_samples += distribution.non_finite_sample_count;
        self.land_samples += audit.land_vertex_samples;
        self.derivative_samples += audit.derivative_samples;
        let derivative_count = count_as_f64(audit.derivative_samples);
        self.slope_gradient_squared +=
            audit.slope_gradient_root_mean_square.powi(2) * derivative_count;
        self.slope_degrees_squared +=
            audit.slope_degrees_root_mean_square.powi(2) * derivative_count;
        self.ruggedness_squared += audit.ruggedness_root_mean_square_m.powi(2) * derivative_count;
        self.extrema_samples += audit.local_extrema_samples;
        Ok(())
    }

    fn finish(self) -> Option<TerrainAggregate> {
        if self.tile_count == 0 || self.elevation_samples == 0 || self.derivative_samples == 0 {
            return None;
        }
        let elevation_count = count_as_f64(self.elevation_samples);
        let derivative_count = count_as_f64(self.derivative_samples);
        Some(TerrainAggregate {
            tile_count: self.tile_count,
            sea_level_m: self
                .sea_level_m
                .expect("a non-empty aggregate has a recorded sea level"),
            elevation_vertex_samples: self.elevation_samples,
            non_finite_elevation_vertex_samples: self.non_finite_samples,
            derivative_samples: self.derivative_samples,
            elevation_minimum_m: self.minimum,
            elevation_maximum_m: self.maximum,
            elevation_vertex_weighted_mean_m: self.mean,
            elevation_vertex_weighted_standard_deviation_m: (self.elevation_m2 / elevation_count)
                .sqrt(),
            land_vertex_weighted_fraction: count_as_f64(self.land_samples) / elevation_count,
            slope_gradient_root_mean_square: (self.slope_gradient_squared / derivative_count)
                .sqrt(),
            slope_degrees_root_mean_square: (self.slope_degrees_squared / derivative_count).sqrt(),
            ruggedness_root_mean_square_m: (self.ruggedness_squared / derivative_count).sqrt(),
            local_extrema_fraction: count_as_f64(self.extrema_samples) / derivative_count,
        })
    }
}

#[allow(clippy::cast_precision_loss)]
fn count_as_f64(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    use worldtools_world::{CubeFace, TerrainGenerator, TerrainSettings, TileId, WorldSeed};

    use crate::audit_terrain_at_sea_level;

    use super::*;

    #[test]
    fn aggregation_is_sample_weighted_across_tiles() {
        let settings = TerrainSettings::default();
        let generator = TerrainGenerator::new(WorldSeed(23), settings);
        let audits = [CubeFace::PositiveX, CubeFace::NegativeX].map(|face| {
            let tile = generator.generate(TileId::root(face));
            audit_terrain_at_sea_level(&tile, settings.planet_radius_m, settings.sea_level_m)
        });
        let aggregate = aggregate_terrain(&audits).unwrap().unwrap();
        assert_eq!(aggregate.tile_count, 2);
        assert_eq!(
            aggregate.elevation_vertex_samples,
            2 * worldtools_world::TILE_SAMPLES * worldtools_world::TILE_SAMPLES
        );
        assert!(aggregate.elevation_minimum_m <= aggregate.elevation_maximum_m);
        assert!((0.0..=1.0).contains(&aggregate.land_vertex_weighted_fraction));
    }

    #[test]
    fn mixed_sea_levels_are_rejected() {
        let settings = TerrainSettings::default();
        let tile = TerrainGenerator::new(WorldSeed(23), settings)
            .generate(TileId::root(CubeFace::PositiveX));
        let first = audit_terrain_at_sea_level(&tile, settings.planet_radius_m, 0.0);
        let second = audit_terrain_at_sea_level(&tile, settings.planet_radius_m, 100.0);
        assert!(matches!(
            aggregate_terrain([&first, &second]),
            Err(TerrainAggregateError::MixedSeaLevels { .. })
        ));
    }
}
