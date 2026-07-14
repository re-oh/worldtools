use rayon::prelude::*;

use crate::{AtlasGrid, stages::math::smoothstep};

/// Surface statistics aggregated from a detailed atlas into a coarser grid.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CoarseSurface {
    pub(crate) mean_elevation_m: Vec<f32>,
    pub(crate) barrier_elevation_m: Vec<f32>,
    pub(crate) land_fraction: Vec<f32>,
}

#[derive(Clone, Copy, Debug)]
struct SurfaceAccumulator {
    weighted_elevation_m: f64,
    total_weight: f64,
    land_weight: f64,
    barrier_elevation_m: f32,
    source_cell_count: u32,
}

impl Default for SurfaceAccumulator {
    fn default() -> Self {
        Self {
            weighted_elevation_m: 0.0,
            total_weight: 0.0,
            land_weight: 0.0,
            barrier_elevation_m: f32::NEG_INFINITY,
            source_cell_count: 0,
        }
    }
}

/// Aggregates every fine cell into exactly one coarse cell.
///
/// Means and land fractions use spherical cell-area weights. The barrier
/// elevation preserves the highest terrain in each footprint so narrow
/// mountain chains remain visible to a coarse atmospheric model.
#[must_use]
#[allow(clippy::cast_possible_truncation)] // Atlas channels intentionally use f32 storage precision.
pub(crate) fn downsample_surface(
    fine_grid: AtlasGrid,
    coarse_grid: AtlasGrid,
    elevation_m: &[f32],
    sea_level_m: f32,
) -> CoarseSurface {
    assert_eq!(elevation_m.len(), fine_grid.len());
    assert!(coarse_grid.width() <= fine_grid.width());
    assert!(coarse_grid.height() <= fine_grid.height());
    assert!(elevation_m.iter().all(|value| value.is_finite()));
    assert!(sea_level_m.is_finite());

    let mut cells = vec![SurfaceAccumulator::default(); coarse_grid.len()];
    for (fine_index, &elevation) in elevation_m.iter().enumerate() {
        let coarse_index = coarse_cell_for(fine_grid, coarse_grid, fine_index);
        let area_weight = fine_grid.point(fine_index).latitude.cos().max(1.0e-6);
        let cell = &mut cells[coarse_index];

        cell.weighted_elevation_m += f64::from(elevation) * area_weight;
        cell.total_weight += area_weight;
        if elevation > sea_level_m {
            cell.land_weight += area_weight;
        }
        cell.barrier_elevation_m = cell.barrier_elevation_m.max(elevation);
        cell.source_cell_count += 1;
    }

    let mut mean_elevation_m = Vec::with_capacity(coarse_grid.len());
    let mut barrier_elevation_m = Vec::with_capacity(coarse_grid.len());
    let mut land_fraction = Vec::with_capacity(coarse_grid.len());
    for cell in cells {
        assert!(cell.source_cell_count > 0);
        mean_elevation_m.push((cell.weighted_elevation_m / cell.total_weight) as f32);
        barrier_elevation_m.push(cell.barrier_elevation_m);
        land_fraction.push((cell.land_weight / cell.total_weight) as f32);
    }

    CoarseSurface {
        mean_elevation_m,
        barrier_elevation_m,
        land_fraction,
    }
}

fn coarse_cell_for(fine_grid: AtlasGrid, coarse_grid: AtlasGrid, fine_index: usize) -> usize {
    let (fine_x, fine_y) = fine_grid.coordinates(fine_index);
    let coarse_x = fine_x * coarse_grid.width() / fine_grid.width();
    let coarse_y = fine_y * coarse_grid.height() / fine_grid.height();
    coarse_grid.index(coarse_x, coarse_y)
}

/// Bilinearly samples one scalar coarse-grid field at every fine cell center.
#[must_use]
pub(crate) fn upsample_scalar(
    coarse_grid: AtlasGrid,
    fine_grid: AtlasGrid,
    coarse_values: &[f32],
) -> Vec<f32> {
    assert_eq!(coarse_values.len(), coarse_grid.len());
    assert!(coarse_values.iter().all(|value| value.is_finite()));

    (0..fine_grid.len())
        .into_par_iter()
        .map(|index| coarse_grid.sample_scalar(coarse_values, fine_grid.point(index)))
        .collect()
}

/// Applies a local lapse rate relative to an interpolated coarse elevation.
///
/// This is separated from resampling so ocean moderation and other model-
/// specific adjustments can be composed before or after it.
#[must_use]
pub(crate) fn apply_lapse_rate(
    temperature_c: &[f32],
    fine_elevation_m: &[f32],
    reference_elevation_m: &[f32],
    lapse_rate_c_per_m: f32,
) -> Vec<f32> {
    assert_aligned_finite(&[temperature_c, fine_elevation_m, reference_elevation_m]);
    assert!(lapse_rate_c_per_m.is_finite() && lapse_rate_c_per_m >= 0.0);

    temperature_c
        .par_iter()
        .zip(fine_elevation_m)
        .zip(reference_elevation_m)
        .map(|((&temperature, &fine_elevation), &reference_elevation)| {
            let elevation_delta = (fine_elevation - reference_elevation).clamp(-3_000.0, 9_000.0);
            (temperature - elevation_delta * lapse_rate_c_per_m).clamp(-95.0, 65.0)
        })
        .collect()
}

/// Applies a bounded windward enhancement and leeward rain shadow.
///
/// Wind components and precipitation are expected to have already been
/// interpolated onto `fine_grid`. Terrain response is based on physical slope,
/// which keeps the correction stable when atlas resolution changes.
#[must_use]
pub(crate) fn apply_orographic_precipitation(
    fine_grid: AtlasGrid,
    planet_radius_m: f64,
    precipitation_mm: &[f32],
    fine_elevation_m: &[f32],
    wind_east: &[f32],
    wind_north: &[f32],
) -> Vec<f32> {
    assert_aligned_finite(&[precipitation_mm, fine_elevation_m, wind_east, wind_north]);
    assert_eq!(precipitation_mm.len(), fine_grid.len());
    assert!(planet_radius_m.is_finite() && planet_radius_m > 0.0);

    (0..fine_grid.len())
        .into_par_iter()
        .map(|index| {
            let east = wind_east[index];
            let north = wind_north[index];
            let speed = east.hypot(north);
            if speed <= 1.0e-4 {
                return precipitation_mm[index].clamp(0.0, 20_000.0);
            }

            let (x, y) = fine_grid.coordinates(index);
            let x = isize::try_from(x).expect("atlas x fits isize");
            let y = isize::try_from(y).expect("atlas y fits isize");
            let east_step = isize::from(east > 0.0) - isize::from(east < 0.0);
            let north_step = isize::from(north > 0.0) - isize::from(north < 0.0);
            let upwind_zonal = fine_grid.wrapped_index(x - east_step, y);
            let upwind_meridional = fine_grid.wrapped_index(x, y + north_step);
            let east_weight = east.abs();
            let north_weight = north.abs();
            let weight_sum = east_weight + north_weight;
            let upwind_elevation = (fine_elevation_m[upwind_zonal] * east_weight
                + fine_elevation_m[upwind_meridional] * north_weight)
                / weight_sum;
            let (dx, dy) = fine_grid.cell_metrics_m(index, planet_radius_m);
            let travel_m = (dx * east_weight + dy * north_weight) / weight_sum;
            let terrain_gradient = (fine_elevation_m[index] - upwind_elevation) / travel_m.max(1.0);
            let windward = smoothstep(0.000_4, 0.020, terrain_gradient);
            let leeward = smoothstep(0.000_5, 0.018, -terrain_gradient);
            let circulation = smoothstep(0.5, 12.0, speed);
            let multiplier = (1.0 + windward * circulation * 0.62 - leeward * circulation * 0.36)
                .clamp(0.58, 1.72);
            (precipitation_mm[index] * multiplier).clamp(0.0, 20_000.0)
        })
        .collect()
}

fn assert_aligned_finite(fields: &[&[f32]]) {
    let Some(first) = fields.first() else {
        return;
    };
    assert!(
        fields
            .iter()
            .all(|field| field.len() == first.len() && field.iter().all(|value| value.is_finite()))
    );
}

#[cfg(test)]
mod tests {
    use worldtools_world::GeoPoint;

    use super::*;

    #[test]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn uneven_downsample_assigns_every_source_cell_once() {
        let fine = AtlasGrid::new(7, 5);
        let coarse = AtlasGrid::new(3, 2);
        let elevation = (0..fine.len())
            .map(|index| fine.coordinates(index).0 as f32 * 100.0)
            .collect::<Vec<_>>();
        let surface = downsample_surface(fine, coarse, &elevation, 250.0);
        let mut source_cell_count = vec![0_u32; coarse.len()];
        for fine_index in 0..fine.len() {
            source_cell_count[coarse_cell_for(fine, coarse, fine_index)] += 1;
        }

        assert_eq!(source_cell_count.iter().sum::<u32>(), 35);
        assert_eq!(source_cell_count, vec![9, 6, 6, 6, 4, 4]);
        for y in 0..coarse.height() {
            assert!((surface.mean_elevation_m[coarse.index(0, y)] - 100.0).abs() < 0.001);
            assert!((surface.mean_elevation_m[coarse.index(1, y)] - 350.0).abs() < 0.001);
            assert!((surface.mean_elevation_m[coarse.index(2, y)] - 550.0).abs() < 0.001);
            assert!((surface.barrier_elevation_m[coarse.index(0, y)] - 200.0).abs() < 0.001);
            assert!((surface.barrier_elevation_m[coarse.index(1, y)] - 400.0).abs() < 0.001);
            assert!((surface.barrier_elevation_m[coarse.index(2, y)] - 600.0).abs() < 0.001);
            assert!(surface.land_fraction[coarse.index(0, y)].abs() < 0.001);
            assert!((surface.land_fraction[coarse.index(1, y)] - 1.0).abs() < 0.001);
            assert!((surface.land_fraction[coarse.index(2, y)] - 1.0).abs() < 0.001);
        }
        assert!(surface.land_fraction.iter().all(|value| value.is_finite()));
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn upsampling_is_finite_and_continuous_at_the_antimeridian() {
        let coarse = AtlasGrid::new(16, 8);
        let fine = AtlasGrid::new(70, 35);
        let values = (0..coarse.len())
            .map(|index| coarse.point(index).longitude.cos() as f32)
            .collect::<Vec<_>>();
        let upsampled = upsample_scalar(coarse, fine, &values);

        assert!(upsampled.iter().all(|value| value.is_finite()));
        let west = fine.sample_scalar(&upsampled, GeoPoint::from_degrees(0.0, -179.999));
        let east = fine.sample_scalar(&upsampled, GeoPoint::from_degrees(0.0, 179.999));
        assert!((west - east).abs() < 0.001);
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn fine_climate_corrections_remain_finite_and_bounded() {
        let grid = AtlasGrid::new(36, 18);
        let count = grid.len();
        let elevation = (0..count)
            .map(|index| grid.coordinates(index).0 as f32 * 800.0)
            .collect::<Vec<_>>();
        let reference = vec![500.0; count];
        let temperature = apply_lapse_rate(&vec![18.0; count], &elevation, &reference, 0.0063);
        let precipitation = apply_orographic_precipitation(
            grid,
            6_371_000.0,
            &vec![900.0; count],
            &elevation,
            &vec![8.0; count],
            &vec![0.0; count],
        );

        assert!(temperature.iter().all(|value| value.is_finite()));
        assert!(
            precipitation
                .iter()
                .all(|value| value.is_finite() && (0.0..=20_000.0).contains(value))
        );
        assert!(precipitation.iter().any(|value| *value > 900.0));
        assert!(precipitation.iter().any(|value| *value < 900.0));
    }
}
