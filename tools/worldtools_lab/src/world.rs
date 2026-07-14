use std::{collections::BTreeMap, time::Instant};

use anyhow::Result;
use serde::Serialize;
use worldtools_analysis::{WorldSimulationAudit, audit_world_snapshot};
use worldtools_simulation::{SimulationSettings, WorldSnapshot};
use worldtools_world::{TerrainSettings, WorldSeed};

use crate::{args::WorldArgs, report};

#[derive(Debug, Serialize)]
struct WorldReport {
    seed: u64,
    fingerprint: String,
    generation_ms: f64,
    settings: SimulationSettings,
    land_cells: usize,
    ocean_cells: usize,
    ranges: BTreeMap<String, NumericRange>,
    soil_classes: BTreeMap<String, usize>,
    biomes: BTreeMap<String, usize>,
    lithologies: BTreeMap<String, usize>,
    deposits: BTreeMap<String, usize>,
    audit: WorldSimulationAudit,
}

#[derive(Clone, Copy, Debug, Serialize)]
struct NumericRange {
    minimum: f32,
    maximum: f32,
    mean: f64,
}

#[derive(Clone, Copy, Debug)]
struct RangeAccumulator {
    minimum: f32,
    maximum: f32,
    sum: f64,
    count: usize,
}

impl Default for RangeAccumulator {
    fn default() -> Self {
        Self {
            minimum: f32::INFINITY,
            maximum: f32::NEG_INFINITY,
            sum: 0.0,
            count: 0,
        }
    }
}

impl RangeAccumulator {
    fn observe(&mut self, value: f32) {
        self.minimum = self.minimum.min(value);
        self.maximum = self.maximum.max(value);
        self.sum += f64::from(value);
        self.count += 1;
    }

    fn finish(self) -> NumericRange {
        let count = f64::from(u32::try_from(self.count).expect("atlas cell count fits u32"));
        NumericRange {
            minimum: self.minimum,
            maximum: self.maximum,
            mean: self.sum / count,
        }
    }
}

pub fn world(arguments: &WorldArgs) -> Result<()> {
    let settings = SimulationSettings {
        atlas_width: arguments.width,
        atlas_height: arguments.height,
        climate_width: (arguments.width / 4).max(16),
        climate_height: (arguments.height / 4).max(8),
        plate_count: arguments.plates,
        hotspot_count: arguments.hotspots,
        geological_age_myr: arguments.geological_age_myr,
        erosion_iterations: arguments.erosion_iterations,
        moisture_iterations: arguments.moisture_iterations,
        glacial_iterations: 8,
    };
    let started = Instant::now();
    let snapshot = WorldSnapshot::generate(
        WorldSeed(arguments.seed),
        TerrainSettings::default(),
        settings,
    );
    let generation_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let summary = summarize(arguments.seed, generation_ms, &snapshot);
    report::write_json(&summary, arguments.output.as_deref())
}

fn summarize(seed: u64, generation_ms: f64, snapshot: &WorldSnapshot) -> WorldReport {
    let mut land_cells = 0;
    let mut ocean_cells = 0;
    let mut soil_classes = BTreeMap::new();
    let mut biomes = BTreeMap::new();
    let mut lithologies = BTreeMap::new();
    let mut deposits = BTreeMap::new();
    let mut ranges = (0..8)
        .map(|_| RangeAccumulator::default())
        .collect::<Vec<_>>();

    for index in 0..snapshot.grid().len() {
        let sample = snapshot.sample(snapshot.grid().point(index));
        let ocean = sample.elevation_m <= snapshot.terrain_settings().sea_level_m;
        land_cells += usize::from(!ocean);
        ocean_cells += usize::from(ocean);
        increment(&mut soil_classes, sample.soil.kind.label());
        increment(&mut biomes, sample.vegetation.biome.label());
        increment(&mut lithologies, sample.geology.lithology.label());
        increment(&mut deposits, sample.resources.dominant.label());
        for (range, value) in ranges.iter_mut().zip([
            sample.elevation_m,
            sample.climate.temperature_c,
            sample.climate.precipitation_mm,
            sample.hydrology.river_strength,
            sample.hydrology.erosion_m,
            sample.geology.sediment_m,
            sample.geology.volcanic_ash_m,
            sample.resources.richness,
        ]) {
            range.observe(value);
        }
    }

    let names = [
        "elevation_m",
        "temperature_c",
        "precipitation_mm",
        "river_strength",
        "erosion_m",
        "sediment_m",
        "volcanic_ash_m",
        "resource_richness",
    ];
    WorldReport {
        seed,
        fingerprint: fingerprint_hex(snapshot.fingerprint()),
        generation_ms,
        settings: snapshot.simulation_settings(),
        land_cells,
        ocean_cells,
        ranges: names
            .into_iter()
            .zip(ranges.into_iter().map(RangeAccumulator::finish))
            .map(|(name, range)| (name.to_owned(), range))
            .collect(),
        soil_classes,
        biomes,
        lithologies,
        deposits,
        audit: audit_world_snapshot(snapshot),
    }
}

fn increment(counts: &mut BTreeMap<String, usize>, label: &str) {
    *counts.entry(label.to_owned()).or_default() += 1;
}

fn fingerprint_hex(fingerprint: [u8; 32]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(64);
    for byte in fingerprint {
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_categories_cover_the_complete_atlas() {
        let settings = SimulationSettings {
            atlas_width: 32,
            atlas_height: 16,
            climate_width: 16,
            climate_height: 8,
            plate_count: 4,
            hotspot_count: 1,
            geological_age_myr: 40,
            erosion_iterations: 1,
            moisture_iterations: 4,
            glacial_iterations: 1,
        };
        let snapshot = WorldSnapshot::generate(WorldSeed(4), TerrainSettings::default(), settings);
        let summary = summarize(4, 0.0, &snapshot);
        let cells = snapshot.grid().len();

        assert_eq!(summary.land_cells + summary.ocean_cells, cells);
        assert_eq!(summary.biomes.values().sum::<usize>(), cells);
        assert_eq!(summary.deposits.values().sum::<usize>(), cells);
    }
}
