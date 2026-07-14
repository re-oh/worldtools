use serde::{Deserialize, Serialize};
use worldtools_world::{TILE_CELLS, TILE_SAMPLES, TerrainTile, angular_distance};

use crate::Distribution;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainAudit {
    pub elevation: Distribution,
    pub sea_level_m: f32,
    pub land_vertex_samples: usize,
    pub land_vertex_fraction: f64,
    pub derivative_samples: usize,
    pub slope_gradient_root_mean_square: f64,
    pub slope_degrees_root_mean_square: f64,
    pub ruggedness_root_mean_square_m: f64,
    pub local_extrema_samples: usize,
    pub local_extrema_fraction: f64,
}

/// Audits one tile using its exact cube-sphere sample spacing.
///
/// Elevation and land fractions are vertex-sample weighted, not surface-area
/// weighted. Shared tile boundary vertices are present in each tile report.
///
/// # Panics
/// Panics when `planet_radius_m` is not finite and greater than zero.
#[must_use]
pub fn audit_terrain_at_sea_level(
    tile: &TerrainTile,
    planet_radius_m: f64,
    sea_level_m: f32,
) -> TerrainAudit {
    assert!(planet_radius_m.is_finite() && planet_radius_m > 0.0);
    let mut interior = Vec::with_capacity(TILE_SAMPLES * TILE_SAMPLES);
    for y in 0..TILE_SAMPLES {
        for x in 0..TILE_SAMPLES {
            interior.push(tile.interior_sample(x, y));
        }
    }

    let mut slope_squared = 0.0_f64;
    let mut slope_degrees_squared = 0.0_f64;
    let mut ruggedness_squared = 0.0_f64;
    let mut extrema = 0_usize;
    let mut derivative_samples = 0_usize;

    for y in 1..TILE_CELLS {
        for x in 1..TILE_CELLS {
            let center = tile.interior_sample(x, y);
            let west = tile.interior_sample(x - 1, y);
            let east = tile.interior_sample(x + 1, y);
            let north = tile.interior_sample(x, y - 1);
            let south = tile.interior_sample(x, y + 1);
            let east_west_distance = planet_radius_m
                * angular_distance(
                    tile.id.sample_direction(x - 1, y),
                    tile.id.sample_direction(x + 1, y),
                );
            let north_south_distance = planet_radius_m
                * angular_distance(
                    tile.id.sample_direction(x, y - 1),
                    tile.id.sample_direction(x, y + 1),
                );
            let east_west_gradient = f64::from(east - west) / east_west_distance;
            let north_south_gradient = f64::from(south - north) / north_south_distance;
            let gradient = east_west_gradient
                .mul_add(
                    east_west_gradient,
                    north_south_gradient * north_south_gradient,
                )
                .sqrt();
            slope_squared += gradient * gradient;
            let slope_degrees = gradient.atan().to_degrees();
            slope_degrees_squared += slope_degrees * slope_degrees;

            let mut lower = true;
            let mut higher = true;
            let mut neighborhood_squared = 0.0_f64;
            for neighbor_y in (y - 1)..=(y + 1) {
                for neighbor_x in (x - 1)..=(x + 1) {
                    if neighbor_x == x && neighbor_y == y {
                        continue;
                    }
                    let neighbor = tile.interior_sample(neighbor_x, neighbor_y);
                    lower &= center < neighbor;
                    higher &= center > neighbor;
                    let delta = f64::from(center - neighbor);
                    neighborhood_squared += delta * delta;
                }
            }
            ruggedness_squared += neighborhood_squared / 8.0;
            extrema += usize::from(lower || higher);
            derivative_samples += 1;
        }
    }

    let land_vertex_samples = interior
        .iter()
        .filter(|&&height| height >= sea_level_m)
        .count();
    let elevation_samples = count_as_f64(interior.len());
    let derivative_count = count_as_f64(derivative_samples);
    TerrainAudit {
        elevation: Distribution::measure(&interior),
        sea_level_m,
        land_vertex_samples,
        land_vertex_fraction: count_as_f64(land_vertex_samples) / elevation_samples,
        derivative_samples,
        slope_gradient_root_mean_square: (slope_squared / derivative_count).sqrt(),
        slope_degrees_root_mean_square: (slope_degrees_squared / derivative_count).sqrt(),
        ruggedness_root_mean_square_m: (ruggedness_squared / derivative_count).sqrt(),
        local_extrema_samples: extrema,
        local_extrema_fraction: count_as_f64(extrema) / derivative_count,
    }
}

/// Convenience audit for worlds whose sea level is zero metres.
///
/// # Panics
/// Panics when `planet_radius_m` is not finite and greater than zero.
#[must_use]
pub fn audit_terrain(tile: &TerrainTile, planet_radius_m: f64) -> TerrainAudit {
    audit_terrain_at_sea_level(tile, planet_radius_m, 0.0)
}

#[allow(clippy::cast_precision_loss)]
fn count_as_f64(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    use worldtools_world::{CubeFace, TerrainGenerator, TerrainSettings, TileId, WorldSeed};

    use super::*;

    #[test]
    fn audit_reports_complete_sample_basis() {
        let settings = TerrainSettings::default();
        let tile = TerrainGenerator::new(WorldSeed(7), settings)
            .generate(TileId::root(CubeFace::PositiveX));
        let audit =
            audit_terrain_at_sea_level(&tile, settings.planet_radius_m, settings.sea_level_m);
        assert_eq!(audit.elevation.sample_count, TILE_SAMPLES * TILE_SAMPLES);
        assert_eq!(
            audit.derivative_samples,
            (TILE_CELLS - 1) * (TILE_CELLS - 1)
        );
        assert!(audit.slope_degrees_root_mean_square.is_finite());
        assert!((0.0..=1.0).contains(&audit.land_vertex_fraction));
    }
}
