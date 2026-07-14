use worldtools_world::TerrainSettings;

use crate::{
    AtlasGrid, SimulationSettings,
    stages::{
        climate::ClimateState, hydrology::HydrologyState, math::smoothstep,
        tectonics::TectonicState,
    },
};

const NO_FLOW: usize = usize::MAX;

#[derive(Debug)]
pub(crate) struct GlacialState {
    pub(crate) maximum_ice_fraction: Vec<f32>,
    pub(crate) ice_flux: Vec<f32>,
    pub(crate) erosion_m: Vec<f32>,
    pub(crate) till_m: Vec<f32>,
    pub(crate) outwash_m: Vec<f32>,
    pub(crate) rebound_m: Vec<f32>,
}

/// Applies one deterministic, long-timescale glacial advance and retreat to the
/// tectonic surface. A later hydrology refresh is expected to reroute water over
/// the modified elevation field.
#[allow(clippy::too_many_lines)]
pub(crate) fn simulate(
    grid: AtlasGrid,
    terrain: TerrainSettings,
    settings: SimulationSettings,
    tectonics: &mut TectonicState,
    climate: &ClimateState,
    hydrology: &mut HydrologyState,
) -> GlacialState {
    debug_assert_eq!(tectonics.elevation_m.len(), grid.len());
    debug_assert_eq!(tectonics.lithology.len(), grid.len());
    debug_assert_eq!(climate.temperature_c.len(), grid.len());
    debug_assert_eq!(climate.precipitation_mm.len(), grid.len());
    debug_assert_eq!(climate.seasonality.len(), grid.len());
    debug_assert_eq!(hydrology.runoff.len(), grid.len());
    debug_assert_eq!(hydrology.river_strength.len(), grid.len());
    debug_assert_eq!(hydrology.wetness.len(), grid.len());
    debug_assert_eq!(hydrology.erosion_m.len(), grid.len());
    debug_assert_eq!(hydrology.sediment_m.len(), grid.len());

    let mut maximum_ice_fraction = (0..grid.len())
        .map(|index| {
            if tectonics.elevation_m[index] <= terrain.sea_level_m {
                return 0.0;
            }
            let (winter_temperature, summer_temperature) =
                seasonal_temperatures(climate.temperature_c[index], climate.seasonality[index]);
            // The history pass represents a colder glacial maximum rather than
            // claiming that the present annual climate is permanently glaciated.
            let glacial_summer = summer_temperature - 9.0;
            let winter_snow = 1.0 - smoothstep(-8.0, 2.0, winter_temperature);
            let summer_survival = 1.0 - smoothstep(-3.0, 5.0, glacial_summer);
            let snow_supply = smoothstep(90.0, 1_800.0, climate.precipitation_mm[index]);
            (winter_snow * summer_survival * (0.28 + snow_supply * 0.72)).clamp(0.0, 1.0)
        })
        .collect::<Vec<_>>();

    let downstream = downhill_field(grid, &tectonics.elevation_m, terrain.sea_level_m);
    let raw_flux =
        accumulate_downhill_flux(&tectonics.elevation_m, &maximum_ice_fraction, &downstream);
    let max_flux = raw_flux.iter().copied().fold(1.0_f32, f32::max);
    let log_max_flux = max_flux.ln_1p();
    let ice_flux = raw_flux
        .iter()
        .map(|flux| (flux.ln_1p() / log_max_flux).clamp(0.0, 1.0))
        .collect::<Vec<_>>();
    for index in 0..grid.len() {
        if tectonics.elevation_m[index] > terrain.sea_level_m {
            // Routed ice extends valley glaciers below their local accumulation
            // zone while retaining the climatic maximum on ice-sheet interiors.
            maximum_ice_fraction[index] = maximum_ice_fraction[index].max(ice_flux[index] * 0.78);
        }
    }

    let history_scale = (f32::from(settings.geological_age_myr) / 240.0)
        .sqrt()
        .clamp(0.35, 1.8);
    let integration_scale = (f32::from(settings.glacial_iterations) / 8.0)
        .sqrt()
        .clamp(0.45, 1.65);
    let erosion_m = (0..grid.len())
        .map(|index| {
            if maximum_ice_fraction[index] <= 0.0 {
                return 0.0;
            }
            let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
            let slope_work = 0.18 + smoothstep(0.000_04, 0.035, slope) * 0.82;
            let erodibility = 1.12 - lithology_hardness(tectonics.lithology[index]);
            (maximum_ice_fraction[index]
                * ice_flux[index]
                * slope_work
                * (0.32 + erodibility)
                * 310.0
                * history_scale
                * integration_scale)
                .clamp(0.0, 420.0)
        })
        .collect::<Vec<_>>();

    let mut till_m = vec![0.0_f32; grid.len()];
    let mut outwash_m = vec![0.0_f32; grid.len()];
    let mut meltwater = vec![0.0_f32; grid.len()];
    for index in 0..grid.len() {
        let ice = maximum_ice_fraction[index];
        if ice <= 0.02 || erosion_m[index] <= 0.0 {
            continue;
        }
        let target = downstream[index];
        let target_ice = if target == NO_FLOW {
            0.0
        } else {
            maximum_ice_fraction[target]
        };
        let reaches_ocean =
            target != NO_FLOW && tectonics.elevation_m[target] <= terrain.sea_level_m;
        let margin = target == NO_FLOW || reaches_ocean || target_ice < ice * 0.58;
        if !margin {
            continue;
        }

        let deposit_at = if target == NO_FLOW || reaches_ocean {
            index
        } else {
            target
        };
        let (_, summer_temperature) =
            seasonal_temperatures(climate.temperature_c[index], climate.seasonality[index]);
        let melt = smoothstep(-4.0, 9.0, summer_temperature) * ice;
        till_m[deposit_at] += erosion_m[index] * (0.34 + (1.0 - melt) * 0.20);
        outwash_m[deposit_at] += erosion_m[index] * melt * 0.26;
        meltwater[deposit_at] = (meltwater[deposit_at] + melt * 0.42).min(1.0);

        let outwash_target = downstream[deposit_at];
        if outwash_target != NO_FLOW && tectonics.elevation_m[outwash_target] > terrain.sea_level_m
        {
            outwash_m[outwash_target] += erosion_m[index] * melt * 0.10;
            meltwater[outwash_target] = (meltwater[outwash_target] + melt * 0.24).min(1.0);
        }
    }
    for value in &mut till_m {
        *value = value.min(95.0);
    }
    for value in &mut outwash_m {
        *value = value.min(70.0);
    }

    let rebound_m = erosion_m
        .iter()
        .zip(&maximum_ice_fraction)
        .map(|(&erosion, &ice)| (erosion * 0.18 + ice * 18.0).clamp(0.0, 88.0))
        .collect::<Vec<_>>();

    for index in 0..grid.len() {
        let deposited = till_m[index] + outwash_m[index];
        tectonics.elevation_m[index] += -erosion_m[index] + rebound_m[index] + deposited;
        tectonics.sediment_m[index] = (tectonics.sediment_m[index] + deposited).clamp(0.0, 240.0);
        hydrology.erosion_m[index] += erosion_m[index];
        hydrology.sediment_m[index] += deposited;
        hydrology.runoff[index] =
            (hydrology.runoff[index] + meltwater[index] * 0.18).clamp(0.0, 1.0);
        hydrology.wetness[index] = hydrology.wetness[index]
            .max(meltwater[index] * 0.62)
            .clamp(0.0, 1.0);
        hydrology.river_strength[index] = hydrology.river_strength[index]
            .max(meltwater[index] * 0.30)
            .clamp(0.0, 1.0);
    }

    GlacialState {
        maximum_ice_fraction,
        ice_flux,
        erosion_m,
        till_m,
        outwash_m,
        rebound_m,
    }
}

fn seasonal_temperatures(annual_temperature_c: f32, seasonality: f32) -> (f32, f32) {
    let amplitude = 3.5 + seasonality.clamp(0.0, 1.0) * 13.5;
    (
        annual_temperature_c - amplitude,
        annual_temperature_c + amplitude,
    )
}

fn downhill_field(grid: AtlasGrid, elevation_m: &[f32], sea_level_m: f32) -> Vec<usize> {
    (0..grid.len())
        .map(|index| {
            if elevation_m[index] <= sea_level_m {
                return NO_FLOW;
            }
            grid.neighbors8(index)
                .into_iter()
                .filter(|&neighbor| elevation_m[neighbor] < elevation_m[index])
                .min_by(|&left, &right| {
                    elevation_m[left]
                        .total_cmp(&elevation_m[right])
                        .then_with(|| left.cmp(&right))
                })
                .unwrap_or(NO_FLOW)
        })
        .collect()
}

fn accumulate_downhill_flux(
    elevation_m: &[f32],
    local_ice: &[f32],
    downstream: &[usize],
) -> Vec<f32> {
    let mut order = (0..elevation_m.len()).collect::<Vec<_>>();
    order.sort_unstable_by(|&left, &right| {
        elevation_m[right]
            .total_cmp(&elevation_m[left])
            .then_with(|| right.cmp(&left))
    });
    let mut flux = local_ice.to_vec();
    for index in order {
        let target = downstream[index];
        if target != NO_FLOW {
            flux[target] += flux[index] * 0.93;
        }
    }
    flux
}

fn lithology_hardness(lithology: u8) -> f32 {
    match lithology {
        0 => 0.82, // Oceanic basalt
        1 => 0.92, // Felsic craton
        2 => 0.38, // Sedimentary rock
        3 => 0.72, // Volcanic arc
        4 => 1.00, // Plutonic rock
        5 => 0.88, // Metamorphic rock
        6 => 0.50, // Carbonate platform
        7 => 0.14, // Unconsolidated sediment
        _ => 0.65,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasonal_temperature_range_grows_with_seasonality() {
        let maritime = seasonal_temperatures(8.0, 0.1);
        let continental = seasonal_temperatures(8.0, 0.9);
        assert!(maritime.0 < 8.0 && maritime.1 > 8.0);
        assert!(continental.0 < maritime.0);
        assert!(continental.1 > maritime.1);
    }

    #[test]
    fn downhill_flux_accumulates_without_uphill_transfer() {
        let grid = AtlasGrid::new(5, 5);
        let mut elevation = vec![400.0; grid.len()];
        let center = grid.index(2, 2);
        let lower = grid.index(3, 2);
        let outlet = grid.index(4, 2);
        elevation[center] = 300.0;
        elevation[lower] = 200.0;
        elevation[outlet] = 0.0;
        let downstream = downhill_field(grid, &elevation, -10.0);
        assert_eq!(downstream[center], lower);
        assert_eq!(downstream[lower], outlet);

        let mut local_ice = vec![0.0; grid.len()];
        local_ice[center] = 1.0;
        let flux = accumulate_downhill_flux(&elevation, &local_ice, &downstream);
        assert!(flux[lower] > 0.9);
        assert!(flux[outlet] > 0.8);
    }

    #[test]
    fn hardness_proxy_keeps_unconsolidated_material_most_erodible() {
        assert!(lithology_hardness(7) < lithology_hardness(2));
        assert!(lithology_hardness(2) < lithology_hardness(0));
        assert!(lithology_hardness(0) < lithology_hardness(4));
    }
}
