use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use anyhow::{Context, Result, ensure};
use rayon::prelude::*;
use serde::Serialize;
use worldtools_analysis::{
    EdgeDirection, TerrainAggregate, aggregate_terrain, audit_child_consistency,
    audit_same_face_seam, audit_terrain_at_sea_level, audit_tile_seam,
};
use worldtools_world::{TerrainGenerator, TerrainSettings, TerrainTile, TileId, WorldSeed};

use crate::{
    args::VerifyArgs,
    continuity::{ContinuityReport, ErrorAccumulator},
    cube_edges::{CUBE_EDGE_RELATIONS, edge_tile},
    report::write_json,
    tile_set::{all_tiles, tiles_per_face},
};

const MAX_EXHAUSTIVE_LEVEL: u8 = 3;

#[derive(Debug, Serialize)]
struct VerificationReport {
    schema_version: u8,
    seed: u64,
    level: u8,
    coverage: VerificationCoverage,
    performance: GenerationPerformance,
    seams: SeamVerification,
    lod: ContinuityReport,
    terrain: TerrainAggregate,
}

#[derive(Debug, Serialize)]
struct VerificationCoverage {
    exhaustive: bool,
    cube_faces: usize,
    tiles_per_face: usize,
    generated_tiles: usize,
    elevation_sample_basis: &'static str,
    derivative_sample_basis: &'static str,
    surface_area_weighted: bool,
}

#[derive(Debug, Serialize)]
struct GenerationPerformance {
    generated_tiles: usize,
    elapsed_milliseconds: u128,
}

#[derive(Debug, Serialize)]
struct SeamVerification {
    scope: &'static str,
    same_face_edge_pairs: usize,
    cube_face_edge_pairs: usize,
    #[serde(flatten)]
    continuity: ContinuityReport,
}

pub fn verify(arguments: &VerifyArgs) -> Result<()> {
    ensure!(
        arguments.level <= MAX_EXHAUSTIVE_LEVEL,
        "exhaustive verification is capped at level {MAX_EXHAUSTIVE_LEVEL} to stay below its memory budget"
    );
    let seed = WorldSeed(arguments.seed);
    let settings = TerrainSettings::default();
    let generator = TerrainGenerator::new(seed, settings);
    let ids = all_tiles(arguments.level);

    let started = Instant::now();
    let tiles = ids
        .par_iter()
        .map(|&id| generator.generate(id))
        .collect::<Vec<_>>();
    let elapsed_milliseconds = started.elapsed().as_millis();
    let by_id = tile_index(&tiles);

    let seams = audit_all_seams(&tiles, &by_id, arguments.level)?;
    let lod = audit_lod(&tiles, &generator, arguments.level)?;
    let terrain_audits = tiles
        .par_iter()
        .map(|tile| {
            audit_terrain_at_sea_level(tile, settings.planet_radius_m, settings.sea_level_m)
        })
        .collect::<Vec<_>>();
    let terrain = aggregate_terrain(&terrain_audits)?
        .context("terrain aggregation unexpectedly received no samples")?;

    let report = VerificationReport {
        schema_version: 1,
        seed: arguments.seed,
        level: arguments.level,
        coverage: VerificationCoverage {
            exhaustive: true,
            cube_faces: 6,
            tiles_per_face: tiles_per_face(arguments.level),
            generated_tiles: tiles.len(),
            elevation_sample_basis: "tile vertices; shared tile and cube-face boundary vertices are retained",
            derivative_sample_basis: "interior central differences divided by exact great-circle sample distance",
            surface_area_weighted: false,
        },
        performance: GenerationPerformance {
            generated_tiles: tiles.len(),
            elapsed_milliseconds,
        },
        seams,
        lod,
        terrain,
    };
    write_json(&report, arguments.output.as_deref())
}

fn audit_all_seams(
    tiles: &[TerrainTile],
    by_id: &HashMap<TileId, usize>,
    level: u8,
) -> Result<SeamVerification> {
    let extent = 1_u32 << level;
    let mut errors = ErrorAccumulator::default();
    let mut same_face_edge_pairs = 0_usize;

    for tile in tiles {
        let neighbors = [
            (
                EdgeDirection::East,
                (tile.id.x + 1 < extent).then(|| {
                    TileId::new(tile.id.face, tile.id.level, tile.id.x + 1, tile.id.y)
                        .expect("enumerated east neighbor is valid")
                }),
            ),
            (
                EdgeDirection::South,
                (tile.id.y + 1 < extent).then(|| {
                    TileId::new(tile.id.face, tile.id.level, tile.id.x, tile.id.y + 1)
                        .expect("enumerated south neighbor is valid")
                }),
            ),
        ];
        for (direction, neighbor_id) in neighbors {
            let Some(neighbor_id) = neighbor_id else {
                continue;
            };
            let neighbor = tiles
                .get(
                    *by_id
                        .get(&neighbor_id)
                        .context("same-face neighbor was not generated")?,
                )
                .context("same-face neighbor index was invalid")?;
            let audit = audit_same_face_seam(tile, neighbor, direction)
                .context("same-face edge geometry did not align")?;
            errors.add_seam(audit);
            same_face_edge_pairs += 1;
        }
    }

    let mut cube_face_edge_pairs = 0_usize;
    for relation in CUBE_EDGE_RELATIONS {
        for offset in 0..extent {
            let second_offset = if relation.reversed {
                extent - 1 - offset
            } else {
                offset
            };
            let first_id = edge_tile(relation.first_face, relation.first_edge, level, offset);
            let second_id = edge_tile(
                relation.second_face,
                relation.second_edge,
                level,
                second_offset,
            );
            let first = tiles
                .get(
                    *by_id
                        .get(&first_id)
                        .context("first cube-edge tile was not generated")?,
                )
                .context("first cube-edge tile index was invalid")?;
            let second = tiles
                .get(
                    *by_id
                        .get(&second_id)
                        .context("second cube-edge tile was not generated")?,
                )
                .context("second cube-edge tile index was invalid")?;
            let audit = audit_tile_seam(
                first,
                relation.first_edge,
                second,
                relation.second_edge,
                relation.reversed,
            )
            .context("cube-face edge geometry did not align")?;
            errors.add_seam(audit);
            cube_face_edge_pairs += 1;
        }
    }

    Ok(SeamVerification {
        scope: "all same-face adjacencies and all twelve cube-face boundaries",
        same_face_edge_pairs,
        cube_face_edge_pairs,
        continuity: errors.finish(true),
    })
}

fn audit_lod(
    tiles: &[TerrainTile],
    generator: &TerrainGenerator,
    level: u8,
) -> Result<ContinuityReport> {
    if level == 0 {
        return Ok(ErrorAccumulator::default().finish(false));
    }

    let parent_ids = tiles
        .iter()
        .filter_map(|tile| tile.id.parent())
        .collect::<HashSet<_>>();
    let parents = parent_ids
        .par_iter()
        .map(|&id| (id, generator.generate(id)))
        .collect::<HashMap<_, _>>();
    let mut errors = ErrorAccumulator::default();
    for child in tiles {
        let parent_id = child.id.parent().context("non-root tile had no parent")?;
        let parent = parents
            .get(&parent_id)
            .context("LOD parent was not generated")?;
        let audit = audit_child_consistency(parent, child)
            .context("generated parent and child were not related")?;
        errors.add_lod(audit);
    }
    Ok(errors.finish(true))
}

fn tile_index(tiles: &[TerrainTile]) -> HashMap<TileId, usize> {
    tiles
        .iter()
        .enumerate()
        .map(|(index, tile)| (tile.id, index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_zero_audits_all_cube_edges() {
        let settings = TerrainSettings::default();
        let generator = TerrainGenerator::new(WorldSeed(2), settings);
        let tiles = all_tiles(0)
            .into_iter()
            .map(|id| generator.generate(id))
            .collect::<Vec<_>>();
        let report = audit_all_seams(&tiles, &tile_index(&tiles), 0).unwrap();
        assert_eq!(report.same_face_edge_pairs, 0);
        assert_eq!(report.cube_face_edge_pairs, 12);
        assert_eq!(
            report.continuity.maximum_absolute_error_m.to_bits(),
            0.0_f32.to_bits()
        );
    }

    #[test]
    fn level_zero_lod_is_explicitly_not_applicable() {
        let generator = TerrainGenerator::new(WorldSeed(2), TerrainSettings::default());
        let report = audit_lod(&[], &generator, 0).unwrap();
        assert!(!report.applicable);
        assert_eq!(report.compared_samples, 0);
    }
}
