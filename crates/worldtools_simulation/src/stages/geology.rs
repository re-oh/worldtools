use worldtools_world::TerrainSettings;

use crate::{
    AtlasGrid,
    stages::{
        climate::ClimateState, hydrology::HydrologyState, math::smoothstep,
        tectonics::TectonicState,
    },
};

#[derive(Debug)]
pub(crate) struct GeologyState {
    pub(crate) lithology: Vec<u8>,
    pub(crate) rock_age_myr: Vec<f32>,
    pub(crate) sediment_m: Vec<f32>,
    pub(crate) volcanic_ash_m: Vec<f32>,
    pub(crate) weathering: Vec<f32>,
}

pub(crate) fn evolve_surface_geology(
    grid: AtlasGrid,
    terrain: TerrainSettings,
    tectonics: &TectonicState,
    climate: &ClimateState,
    hydrology: &HydrologyState,
) -> GeologyState {
    let volcanic_ash_m = disperse_ash(grid, tectonics, climate);
    let weathering = (0..grid.len())
        .map(|index| {
            if tectonics.elevation_m[index] <= terrain.sea_level_m {
                0.08
            } else {
                let warmth = smoothstep(-4.0, 29.0, climate.temperature_c[index]);
                let moisture = smoothstep(180.0, 2_800.0, climate.precipitation_mm[index]);
                let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
                (warmth * moisture * (1.0 - smoothstep(0.04, 0.24, slope))
                    + volcanic_ash_m[index].min(1.0) * 0.08)
                    .clamp(0.0, 1.0)
            }
        })
        .collect::<Vec<_>>();
    let sediment_m = hydrology
        .sediment_m
        .iter()
        .zip(&hydrology.erosion_m)
        .map(|(&sediment, &erosion)| (sediment + erosion * 0.18).clamp(0.0, 180.0))
        .collect::<Vec<_>>();
    let lithology = (0..grid.len())
        .map(|index| {
            let elevation = tectonics.elevation_m[index];
            if volcanic_ash_m[index] > 1.6 || tectonics.volcanism[index] > 0.56 {
                3
            } else if sediment_m[index] > 55.0 {
                7
            } else if elevation <= terrain.sea_level_m
                && elevation > terrain.sea_level_m - 260.0
                && climate.temperature_c[index] > 18.0
            {
                6
            } else if sediment_m[index] > 18.0 {
                2
            } else {
                tectonics.lithology[index]
            }
        })
        .collect();

    GeologyState {
        lithology,
        rock_age_myr: tectonics.rock_age_myr.clone(),
        sediment_m,
        volcanic_ash_m,
        weathering,
    }
}

fn disperse_ash(grid: AtlasGrid, tectonics: &TectonicState, climate: &ClimateState) -> Vec<f32> {
    let sources = tectonics
        .volcanism
        .iter()
        .map(|&activity| activity.powf(3.2) * 1.8)
        .collect::<Vec<_>>();
    let mut airborne = sources.clone();
    let mut next = vec![0.0_f32; grid.len()];
    let mut settled = tectonics.volcanic_ash_m.clone();

    for _ in 0..14 {
        next.fill(0.0);
        for index in 0..grid.len() {
            let (x, y) = grid.coordinates(index);
            let x = isize::try_from(x).expect("atlas x fits isize");
            let y = isize::try_from(y).expect("atlas y fits isize");
            let east_step = isize::from(climate.wind_east[index] > 0.08)
                - isize::from(climate.wind_east[index] < -0.08);
            let north_step = isize::from(climate.wind_north[index] < -0.08)
                - isize::from(climate.wind_north[index] > 0.08);
            let destination = grid.wrapped_index(x + east_step, y + north_step);
            let washout = smoothstep(600.0, 3_200.0, climate.precipitation_mm[index]) * 0.18;
            let deposit_fraction = 0.07 + washout;
            settled[index] += airborne[index] * deposit_fraction;
            next[destination] += airborne[index] * (1.0 - deposit_fraction) * 0.91;
            next[index] += sources[index] * 0.22;
        }
        std::mem::swap(&mut airborne, &mut next);
    }
    settled
        .into_iter()
        .map(|ash| ash.clamp(0.0, 16.0))
        .collect()
}
