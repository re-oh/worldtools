use std::time::Instant;

use anyhow::{Context, Result, ensure};
use rayon::prelude::*;
use serde::Serialize;
use worldtools_analysis::{TerrainAggregate, aggregate_terrain, audit_terrain_at_sea_level};
use worldtools_world::{TerrainGenerator, TerrainSettings, TileId, WorldSeed};

use crate::{
    args::SweepArgs,
    report::write_json,
    tile_set::{all_tiles, tiles_per_face},
};

const MAX_SWEEP_LEVEL: u8 = 2;
const MAX_TOTAL_GENERATED_TILES: usize = 4_096;

#[derive(Debug, Serialize)]
struct SweepReport {
    schema_version: u8,
    first_seed: u64,
    seed_count: usize,
    level: u8,
    coverage: SweepCoverage,
    elapsed_milliseconds: u128,
    seeds: Vec<SeedSummary>,
}

#[derive(Debug, Serialize)]
struct SweepCoverage {
    exhaustive_per_seed: bool,
    cube_faces_per_seed: usize,
    tiles_per_seed: usize,
    elevation_sample_basis: &'static str,
    surface_area_weighted: bool,
}

#[derive(Debug, Serialize)]
struct SeedSummary {
    seed: u64,
    terrain: TerrainAggregate,
}

pub fn sweep(arguments: &SweepArgs) -> Result<()> {
    ensure!(arguments.count > 0, "count must be greater than zero");
    ensure!(
        arguments.level <= MAX_SWEEP_LEVEL,
        "sweep level is capped at {MAX_SWEEP_LEVEL}"
    );
    let ids = all_tiles(arguments.level);
    let total_tiles = arguments
        .count
        .checked_mul(ids.len())
        .context("sweep tile budget overflowed")?;
    ensure!(
        total_tiles <= MAX_TOTAL_GENERATED_TILES,
        "sweep would generate {total_tiles} tiles; the limit is {MAX_TOTAL_GENERATED_TILES}"
    );
    arguments
        .first_seed
        .checked_add(u64::try_from(arguments.count - 1).context("seed count does not fit u64")?)
        .context("seed range exceeds u64")?;

    let settings = TerrainSettings::default();
    let started = Instant::now();
    let seeds = (0..arguments.count)
        .into_par_iter()
        .map(|offset| {
            let offset = u64::try_from(offset).context("seed offset does not fit u64")?;
            summarize_seed(arguments.first_seed + offset, &ids, settings)
        })
        .collect::<Result<Vec<_>>>()?;
    let elapsed_milliseconds = started.elapsed().as_millis();

    let report = SweepReport {
        schema_version: 1,
        first_seed: arguments.first_seed,
        seed_count: arguments.count,
        level: arguments.level,
        coverage: SweepCoverage {
            exhaustive_per_seed: true,
            cube_faces_per_seed: 6,
            tiles_per_seed: 6 * tiles_per_face(arguments.level),
            elevation_sample_basis: "tile vertices; shared tile and cube-face boundary vertices are retained",
            surface_area_weighted: false,
        },
        elapsed_milliseconds,
        seeds,
    };
    write_json(&report, arguments.output.as_deref())
}

fn summarize_seed(seed: u64, ids: &[TileId], settings: TerrainSettings) -> Result<SeedSummary> {
    let generator = TerrainGenerator::new(WorldSeed(seed), settings);
    let audits = ids
        .iter()
        .map(|&id| {
            let tile = generator.generate(id);
            audit_terrain_at_sea_level(&tile, settings.planet_radius_m, settings.sea_level_m)
        })
        .collect::<Vec<_>>();
    let terrain = aggregate_terrain(&audits)?.context("seed generated no terrain samples")?;
    Ok(SeedSummary { seed, terrain })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_summary_aggregates_all_six_root_faces() {
        let summary = summarize_seed(17, &all_tiles(0), TerrainSettings::default()).unwrap();
        assert_eq!(summary.terrain.tile_count, 6);
        assert!(summary.terrain.elevation_vertex_samples > 6 * 65_000);
    }
}
