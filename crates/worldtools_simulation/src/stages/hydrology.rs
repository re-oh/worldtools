use std::{cmp::Ordering, collections::BinaryHeap};

use worldtools_world::TerrainSettings;

use crate::{
    AtlasGrid, SimulationSettings,
    stages::{climate::ClimateState, math::smoothstep, tectonics::TectonicState},
};

const NO_FLOW: usize = usize::MAX;

struct FlowField {
    to: Vec<usize>,
    accumulation: Vec<f32>,
    fill_depth_m: Vec<f32>,
}

#[derive(Clone, Copy)]
struct FloodCell {
    elevation_m: f32,
    index: usize,
}

impl PartialEq for FloodCell {
    fn eq(&self, other: &Self) -> bool {
        self.elevation_m.to_bits() == other.elevation_m.to_bits() && self.index == other.index
    }
}

impl Eq for FloodCell {}

impl Ord for FloodCell {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .elevation_m
            .total_cmp(&self.elevation_m)
            .then_with(|| other.index.cmp(&self.index))
    }
}

impl PartialOrd for FloodCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub(crate) struct HydrologyState {
    pub(crate) runoff: Vec<f32>,
    pub(crate) river_strength: Vec<f32>,
    pub(crate) wetness: Vec<f32>,
    pub(crate) lake: Vec<f32>,
    pub(crate) erosion_m: Vec<f32>,
    pub(crate) sediment_m: Vec<f32>,
}

#[allow(clippy::too_many_lines)] // Erosion mutates coupled elevation and sediment in one pass.
pub(crate) fn simulate(
    grid: AtlasGrid,
    terrain: TerrainSettings,
    settings: SimulationSettings,
    climate: &ClimateState,
    tectonics: &mut TectonicState,
) -> HydrologyState {
    let runoff = runoff_from_climate(terrain, climate, &tectonics.elevation_m);
    let mut erosion_total = vec![0.0_f32; grid.len()];
    // This layer records sediment mobilized by the history pass, not inherited
    // seafloor cover. Keeping those quantities separate makes deltas and
    // floodplains visible without painting every surface as active deposition.
    let inherited_sediment = tectonics.sediment_m.clone();
    let mut sediment_total = vec![0.0_f32; grid.len()];
    let history_scale = (f32::from(settings.geological_age_myr) / 240.0)
        .sqrt()
        .clamp(0.35, 1.8);

    for iteration in 0..settings.erosion_iterations {
        let flow = route_and_accumulate(
            grid,
            terrain.planet_radius_m,
            terrain.sea_level_m,
            &tectonics.elevation_m,
            &runoff,
        );
        let max_accumulation = flow.accumulation.iter().copied().fold(1.0_f32, f32::max);
        let log_max = max_accumulation.ln_1p();
        let mut elevation_delta = vec![0.0_f32; grid.len()];

        for index in 0..grid.len() {
            if tectonics.elevation_m[index] <= terrain.sea_level_m {
                continue;
            }
            let downstream = flow.to[index];
            let stream = (flow.accumulation[index].ln_1p() / log_max).clamp(0.0, 1.0);
            let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
            let maturity = (f32::from(iteration) + 1.0) / f32::from(settings.erosion_iterations);
            let incision =
                stream.powf(1.65) * smoothstep(0.000_08, 0.035, slope) * (8.0 + maturity * 21.0);
            let sheet_wash = runoff[index] * smoothstep(0.008, 0.12, slope) * 1.4;
            let eroded = ((incision + sheet_wash) * history_scale).min(46.0);
            elevation_delta[index] -= eroded;
            erosion_total[index] += eroded;

            if downstream == NO_FLOW {
                sediment_total[index] += eroded * 0.75;
            } else {
                let downstream_slope =
                    grid.slope(&tectonics.elevation_m, downstream, terrain.planet_radius_m);
                let deposition =
                    eroded * (0.10 + (1.0 - smoothstep(0.000_2, 0.008, downstream_slope)) * 0.46);
                elevation_delta[downstream] += deposition;
                sediment_total[downstream] += deposition;

                if tectonics.elevation_m[downstream] <= terrain.sea_level_m && stream > 0.66 {
                    // Repeated distributary deposition builds a shallow fan instead of a
                    // single coastline spike, approximating delta progradation.
                    let fan_deposit = eroded * stream * 0.045;
                    for fan_cell in grid.neighbors8(downstream) {
                        if tectonics.elevation_m[fan_cell] > terrain.sea_level_m - 420.0 {
                            elevation_delta[fan_cell] += fan_deposit;
                            sediment_total[fan_cell] += fan_deposit;
                        }
                    }
                }
            }
        }

        for (elevation, delta) in tectonics.elevation_m.iter_mut().zip(elevation_delta) {
            *elevation += delta;
        }
    }

    let flow = route_and_accumulate(
        grid,
        terrain.planet_radius_m,
        terrain.sea_level_m,
        &tectonics.elevation_m,
        &runoff,
    );
    let max_accumulation = flow.accumulation.iter().copied().fold(1.0_f32, f32::max);
    let log_max = max_accumulation.ln_1p();
    let river_strength = flow
        .accumulation
        .iter()
        .zip(&tectonics.elevation_m)
        .map(|(&flow, &elevation)| {
            if elevation <= terrain.sea_level_m {
                0.0
            } else {
                major_river_strength(flow, log_max)
            }
        })
        .collect::<Vec<_>>();
    let lake = (0..grid.len())
        .map(|index| {
            if tectonics.elevation_m[index] <= terrain.sea_level_m {
                return 0.0;
            }
            let basin_depth = smoothstep(1.0, 85.0, flow.fill_depth_m[index]);
            let water_supply = smoothstep(0.12, 0.58, flow.accumulation[index].ln_1p() / log_max);
            (basin_depth * water_supply).clamp(0.0, 1.0)
        })
        .collect::<Vec<_>>();
    let wetness = (0..grid.len())
        .map(|index| {
            let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
            (runoff[index] * 0.48
                + river_strength[index] * 0.30
                + lake[index] * 0.62
                + (1.0 - smoothstep(0.001, 0.035, slope)) * runoff[index] * 0.18)
                .clamp(0.0, 1.0)
        })
        .collect();

    for ((target, inherited), deposited) in tectonics
        .sediment_m
        .iter_mut()
        .zip(inherited_sediment)
        .zip(&sediment_total)
    {
        *target = inherited + deposited;
    }
    HydrologyState {
        runoff,
        river_strength,
        wetness,
        lake,
        erosion_m: erosion_total,
        sediment_m: sediment_total,
    }
}

pub(crate) fn refresh_after_climate(
    grid: AtlasGrid,
    terrain: TerrainSettings,
    climate: &ClimateState,
    tectonics: &TectonicState,
    state: &mut HydrologyState,
) {
    state.runoff = runoff_from_climate(terrain, climate, &tectonics.elevation_m);
    let flow = route_and_accumulate(
        grid,
        terrain.planet_radius_m,
        terrain.sea_level_m,
        &tectonics.elevation_m,
        &state.runoff,
    );
    let max_accumulation = flow.accumulation.iter().copied().fold(1.0_f32, f32::max);
    let log_max = max_accumulation.ln_1p();
    for index in 0..grid.len() {
        state.river_strength[index] = if tectonics.elevation_m[index] <= terrain.sea_level_m {
            0.0
        } else {
            major_river_strength(flow.accumulation[index], log_max)
        };
        let basin_depth = smoothstep(1.0, 85.0, flow.fill_depth_m[index]);
        let water_supply = smoothstep(0.12, 0.58, flow.accumulation[index].ln_1p() / log_max);
        state.lake[index] = if tectonics.elevation_m[index] > terrain.sea_level_m {
            basin_depth * water_supply
        } else {
            0.0
        };
        let slope = grid.slope(&tectonics.elevation_m, index, terrain.planet_radius_m);
        state.wetness[index] = (state.runoff[index] * 0.48
            + state.river_strength[index] * 0.30
            + state.lake[index] * 0.62
            + (1.0 - smoothstep(0.001, 0.035, slope)) * state.runoff[index] * 0.18)
            .clamp(0.0, 1.0);
    }
}

fn runoff_from_climate(
    terrain: TerrainSettings,
    climate: &ClimateState,
    elevation_m: &[f32],
) -> Vec<f32> {
    (0..elevation_m.len())
        .map(|index| {
            if elevation_m[index] <= terrain.sea_level_m {
                return 0.0;
            }
            let precipitation = climate.precipitation_mm[index];
            let retained_moisture = (1.0 - climate.aridity[index]).powf(1.35);
            let thaw = 0.42 + smoothstep(-9.0, 5.0, climate.temperature_c[index]) * 0.58;
            ((precipitation / 2_450.0) * (0.12 + retained_moisture * 0.88) * thaw).clamp(0.0, 1.0)
        })
        .collect()
}

fn major_river_strength(accumulation: f32, log_max: f32) -> f32 {
    smoothstep(0.28, 0.82, accumulation.ln_1p() / log_max).powf(1.12)
}

fn route_and_accumulate(
    grid: AtlasGrid,
    planet_radius_m: f64,
    sea_level_m: f32,
    elevation_m: &[f32],
    runoff: &[f32],
) -> FlowField {
    let (filled_elevation, fill_depth_m) = fill_depressions(grid, sea_level_m, elevation_m);
    let mut flow_to = vec![NO_FLOW; grid.len()];
    for index in 0..grid.len() {
        if elevation_m[index] <= sea_level_m {
            continue;
        }
        let (dx, dy) = grid.cell_metrics_m(index, planet_radius_m);
        let neighbors = grid.neighbors8(index);
        let mut best = NO_FLOW;
        let mut best_gradient = 0.0_f32;
        for (neighbor_slot, neighbor) in neighbors.into_iter().enumerate() {
            let drop = filled_elevation[index] - filled_elevation[neighbor];
            let diagonal = matches!(neighbor_slot, 0 | 2 | 5 | 7);
            let distance = if diagonal {
                dx.hypot(dy)
            } else if matches!(neighbor_slot, 3 | 4) {
                dx
            } else {
                dy
            };
            let gradient = drop / distance;
            if gradient > best_gradient {
                best_gradient = gradient;
                best = neighbor;
            }
        }
        flow_to[index] = best;
    }

    let mut order = (0..grid.len()).collect::<Vec<_>>();
    order.sort_unstable_by(|&left, &right| {
        filled_elevation[right]
            .total_cmp(&filled_elevation[left])
            .then_with(|| right.cmp(&left))
    });
    #[allow(clippy::cast_possible_truncation)]
    let mut accumulation = (0..grid.len())
        .map(|index| runoff[index] * grid.point(index).latitude.cos().abs().max(0.02) as f32)
        .collect::<Vec<_>>();
    for index in order {
        let downstream = flow_to[index];
        if downstream != NO_FLOW {
            accumulation[downstream] += accumulation[index];
        }
    }
    FlowField {
        to: flow_to,
        accumulation,
        fill_depth_m,
    }
}

fn fill_depressions(
    grid: AtlasGrid,
    sea_level_m: f32,
    elevation_m: &[f32],
) -> (Vec<f32>, Vec<f32>) {
    let mut filled = elevation_m.to_vec();
    let mut visited = vec![false; grid.len()];
    let mut frontier = BinaryHeap::new();

    for (index, &elevation) in elevation_m.iter().enumerate() {
        if elevation <= sea_level_m {
            visited[index] = true;
            frontier.push(FloodCell {
                elevation_m: elevation,
                index,
            });
        }
    }

    if frontier.is_empty() {
        let lowest = elevation_m
            .iter()
            .enumerate()
            .min_by(|left, right| left.1.total_cmp(right.1))
            .map_or(0, |(index, _)| index);
        visited[lowest] = true;
        frontier.push(FloodCell {
            elevation_m: elevation_m[lowest],
            index: lowest,
        });
    }

    while let Some(cell) = frontier.pop() {
        for neighbor in grid.neighbors8(cell.index) {
            if visited[neighbor] {
                continue;
            }
            visited[neighbor] = true;
            // A small deterministic gradient resolves flats without materially
            // changing the represented surface or introducing drainage cycles.
            filled[neighbor] = elevation_m[neighbor].max(cell.elevation_m + 0.05);
            frontier.push(FloodCell {
                elevation_m: filled[neighbor],
                index: neighbor,
            });
        }
    }

    let fill_depth_m = filled
        .iter()
        .zip(elevation_m)
        .map(|(&filled, &original)| (filled - original).max(0.0))
        .collect();
    (filled, fill_depth_m)
}
