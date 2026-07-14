use rayon::prelude::*;
use worldtools_world::{TerrainSettings, WorldSeed};

use crate::{
    AtlasGrid, SimulationSettings,
    stages::{math::smoothstep, tectonics::TectonicState},
};

#[derive(Debug)]
pub(crate) struct ClimateState {
    pub(crate) zone: Vec<u8>,
    pub(crate) temperature_c: Vec<f32>,
    pub(crate) precipitation_mm: Vec<f32>,
    pub(crate) seasonality: Vec<f32>,
    pub(crate) wind_east: Vec<f32>,
    pub(crate) wind_north: Vec<f32>,
    pub(crate) aridity: Vec<f32>,
}

#[allow(clippy::too_many_lines)] // Atmospheric fields are produced together to preserve their shared circulation state.
pub(crate) fn simulate(
    grid: AtlasGrid,
    _seed: WorldSeed,
    terrain: TerrainSettings,
    settings: SimulationSettings,
    tectonics: &TectonicState,
) -> ClimateState {
    let ocean = tectonics
        .elevation_m
        .iter()
        .map(|&elevation| elevation <= terrain.sea_level_m)
        .collect::<Vec<_>>();
    let continental_distance = continental_distance(grid, &ocean);

    let atmospheric = (0..grid.len())
        .into_par_iter()
        .map(|index| {
            let point = grid.point(index);
            #[allow(clippy::cast_possible_truncation)]
            let latitude = point.latitude as f32;
            let absolute_latitude = latitude.abs();
            let latitude_degrees = absolute_latitude.to_degrees();
            let elevation_above_sea = (tectonics.elevation_m[index] - terrain.sea_level_m).max(0.0);
            let inland = (continental_distance[index] / 24.0).clamp(0.0, 1.0);

            // Annual mean temperature is controlled by insolation, elevation and
            // continentality. The 6.3 K/km lapse rate is deliberately a little
            // lower than the dry adiabatic rate because this atlas represents a
            // long-term, mixed atmosphere.
            let annual_solar = 31.0 - 47.0 * absolute_latitude.sin().powf(2.30);
            let ocean_moderation = if ocean[index] { 2.8 } else { 0.0 };
            let temperature = annual_solar - elevation_above_sea * 0.0063 - inland * 1.8
                + ocean_moderation * smoothstep(0.55, 1.0, absolute_latitude.sin());
            let seasonality = (absolute_latitude.sin().powf(1.18) * (0.58 + inland * 0.38)
                + inland * 0.08)
                .clamp(0.0, 1.0);

            // Smooth three-cell circulation avoids artificial latitude seams while
            // preserving trades, mid-latitude westerlies and polar easterlies.
            let trades = 1.0 - smoothstep(22.0, 35.0, latitude_degrees);
            let westerlies = smoothstep(24.0, 40.0, latitude_degrees)
                * (1.0 - smoothstep(58.0, 72.0, latitude_degrees));
            let polar_easterlies = smoothstep(60.0, 78.0, latitude_degrees);
            let zonal = -trades + westerlies - polar_easterlies * 0.62;
            let hemisphere = latitude.signum();
            #[allow(clippy::cast_possible_truncation)]
            let planetary_wave = (point.longitude * 2.0 + f64::from(latitude) * 1.5).sin() as f32;
            let hadley_meridional = -hemisphere * trades * 0.24;
            let ferrel_meridional = hemisphere * westerlies * 0.12;
            let polar_meridional = -hemisphere * polar_easterlies * 0.10;
            let meridional = hadley_meridional
                + ferrel_meridional
                + polar_meridional
                + planetary_wave * (0.06 + inland * 0.12);
            let topographic_drag = 1.0 / (1.0 + elevation_above_sea / 12_000.0);
            // Annual prevailing winds are physical metres per second. Synoptic
            // gusts are deliberately outside this long-timescale atlas.
            let magnitude = (5.2 + absolute_latitude.cos() * 5.8) * topographic_drag;
            (
                temperature,
                seasonality,
                zonal * magnitude,
                meridional * magnitude * 0.82,
            )
        })
        .collect::<Vec<_>>();

    let temperature_c = atmospheric.iter().map(|cell| cell.0).collect::<Vec<_>>();
    let seasonality = atmospheric.iter().map(|cell| cell.1).collect::<Vec<_>>();
    let wind_east = atmospheric.iter().map(|cell| cell.2).collect::<Vec<_>>();
    let wind_north = atmospheric.iter().map(|cell| cell.3).collect::<Vec<_>>();
    let precipitation_mm = advect_moisture(
        grid,
        settings.moisture_iterations,
        &tectonics.elevation_m,
        terrain.sea_level_m,
        &ocean,
        &temperature_c,
        &wind_east,
        &wind_north,
    );

    let aridity = (0..grid.len())
        .into_par_iter()
        .map(|index| {
            let temperature = temperature_c[index];
            let precipitation = precipitation_mm[index];
            let wind_speed = wind_east[index].hypot(wind_north[index]);
            let potential_evaporation = ((temperature + 8.0).max(0.0) * 43.0 + 210.0)
                * (1.0 + wind_speed * 0.012)
                * (1.0 + seasonality[index] * 0.08);
            (potential_evaporation / (precipitation + potential_evaporation)).clamp(0.0, 1.0)
        })
        .collect::<Vec<_>>();
    let zone = (0..grid.len())
        .into_par_iter()
        .map(|index| {
            if ocean[index] {
                6
            } else if temperature_c[index] < -12.0 {
                0
            } else if temperature_c[index] < 2.0 {
                1
            } else if aridity[index] > 0.72 {
                2
            } else if temperature_c[index] > 23.0 {
                5
            } else if seasonality[index] > 0.62 {
                4
            } else {
                3
            }
        })
        .collect();

    ClimateState {
        zone,
        temperature_c,
        precipitation_mm,
        seasonality,
        wind_east,
        wind_north,
        aridity,
    }
}

fn continental_distance(grid: AtlasGrid, ocean: &[bool]) -> Vec<f32> {
    let mut distance = ocean
        .iter()
        .map(|&is_ocean| if is_ocean { 0.0_f32 } else { 64.0 })
        .collect::<Vec<_>>();
    let mut next = distance.clone();
    for _ in 0..48 {
        for index in 0..grid.len() {
            let nearest = grid
                .cardinal_neighbors(index)
                .into_iter()
                .map(|neighbor| distance[neighbor] + 1.0)
                .fold(distance[index], f32::min);
            next[index] = nearest;
        }
        std::mem::swap(&mut distance, &mut next);
    }
    distance
}

#[allow(clippy::too_many_arguments)] // The transport pass consumes aligned atmospheric fields without owning them.
fn advect_moisture(
    grid: AtlasGrid,
    iterations: u16,
    elevation_m: &[f32],
    sea_level_m: f32,
    ocean: &[bool],
    temperature_c: &[f32],
    wind_east: &[f32],
    wind_north: &[f32],
) -> Vec<f32> {
    let mut moisture = ocean
        .iter()
        .map(|&is_ocean| if is_ocean { 0.82_f32 } else { 0.035 })
        .collect::<Vec<_>>();
    let mut next = vec![0.0_f32; grid.len()];
    let mut precipitation = vec![0.0_f32; grid.len()];

    for _ in 0..iterations {
        for index in 0..grid.len() {
            let (x, y) = grid.coordinates(index);
            let x = isize::try_from(x).expect("atlas x fits isize");
            let y = isize::try_from(y).expect("atlas y fits isize");
            let east_step =
                isize::from(wind_east[index] > 0.0) - isize::from(wind_east[index] < 0.0);
            let north_step =
                isize::from(wind_north[index] > 0.0) - isize::from(wind_north[index] < 0.0);
            let upstream_zonal = grid.wrapped_index(x - east_step, y);
            let upstream_meridional = grid.wrapped_index(x, y + north_step);
            let east_weight = wind_east[index].abs();
            let north_weight = wind_north[index].abs();
            let weight_sum = east_weight + north_weight + 1.5;
            let incoming = (moisture[upstream_zonal] * east_weight
                + moisture[upstream_meridional] * north_weight
                + moisture[index] * 1.5)
                / weight_sum
                * 0.986;

            let source_elevation = (elevation_m[upstream_zonal].max(sea_level_m) * east_weight
                + elevation_m[upstream_meridional].max(sea_level_m) * north_weight
                + elevation_m[index].max(sea_level_m) * 1.5)
                / weight_sum;
            let rise = (elevation_m[index].max(sea_level_m) - source_elevation).max(0.0);
            let orographic = smoothstep(55.0, 1_450.0, rise) * 0.44;
            let source_temperature = (temperature_c[upstream_zonal] * east_weight
                + temperature_c[upstream_meridional] * north_weight
                + temperature_c[index] * 1.5)
                / weight_sum;
            let cooling = smoothstep(0.5, 9.0, source_temperature - temperature_c[index]) * 0.13;

            #[allow(clippy::cast_possible_truncation)]
            let latitude = grid.point(index).latitude.abs() as f32;
            let equatorial_convergence = 1.0 - smoothstep(0.07, 0.30, latitude);
            let subtropical_descent = latitude_band(latitude, 0.24, 0.48, 0.68);
            let storm_track = latitude_band(latitude, 0.52, 0.82, 1.12);
            let convection =
                equatorial_convergence * smoothstep(18.0, 30.0, temperature_c[index]) * 0.13;
            let frontal = storm_track * 0.065;
            let base_condensation =
                (0.018 + convection + frontal) * (1.0 - subtropical_descent * 0.62);
            let condensation = if ocean[index] {
                (base_condensation * 0.72 + cooling).clamp(0.01, 0.32)
            } else {
                (base_condensation + orographic + cooling).clamp(0.008, 0.68)
            };
            let released = incoming * condensation;
            precipitation[index] += released;
            let remaining = (incoming - released).max(0.0);
            next[index] = if ocean[index] {
                let saturation = 0.56 + smoothstep(-4.0, 30.0, temperature_c[index]) * 0.42;
                remaining.max(saturation)
            } else {
                // A modest fraction of rain returns through evapotranspiration;
                // most continues through runoff, storage, or groundwater.
                (remaining + released * 0.17).clamp(0.0, 1.15)
            };
        }
        std::mem::swap(&mut moisture, &mut next);
    }

    let iteration_scale = 1.0 / f32::from(iterations);
    precipitation
        .into_iter()
        .map(|total| (total * iteration_scale * 55_000.0).clamp(12.0, 7_200.0))
        .collect()
}

fn latitude_band(value: f32, low: f32, peak: f32, high: f32) -> f32 {
    smoothstep(low, peak, value) * (1.0 - smoothstep(peak, high, value))
}
