use std::time::Instant;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded};
use serde::Serialize;
use worldtools_analysis::{
    LodAudit, SeamAudit, TerrainAggregate, TileEdge, aggregate_terrain, audit_child_consistency,
    audit_terrain_at_sea_level, audit_tile_seam,
};
use worldtools_ui::{AnalysisIssue, AnalysisSeverity, AnalysisStatus, DocumentStatus};
use worldtools_world::{CubeFace, TerrainGenerator, TerrainSettings, TileId, WorldSeed};

use crate::diagnostics::DiagnosticDirectory;

use super::io::{timestamp_millis, write_json_atomic};

#[derive(Clone, Copy, Debug, Message)]
pub struct AuditRequest;

#[derive(Resource)]
pub struct AuditRuntime {
    sender: Sender<AuditResult>,
    receiver: Receiver<AuditResult>,
    running: bool,
}

impl Default for AuditRuntime {
    fn default() -> Self {
        let (sender, receiver) = bounded(1);
        Self {
            sender,
            receiver,
            running: false,
        }
    }
}

type AuditResult = Result<TerrainAuditReport, String>;

#[derive(Debug, Serialize)]
struct TerrainAuditReport {
    schema: &'static str,
    captured_unix_ms: u128,
    seed: u64,
    duration_ms: f64,
    passed: bool,
    deterministic: bool,
    terrain: TerrainAggregate,
    seam: SeamAudit,
    lod: LodAudit,
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
pub fn start_audits(
    mut requests: MessageReader<AuditRequest>,
    document: Res<DocumentStatus>,
    mut runtime: ResMut<AuditRuntime>,
) {
    for _ in requests.read() {
        if runtime.running {
            tracing::warn!(
                target: "worldtools::audit",
                "terrain audit request ignored because an audit is already running"
            );
            continue;
        }

        runtime.running = true;
        let seed = document.seed;
        let sender = runtime.sender.clone();
        tracing::info!(target: "worldtools::audit", seed, "terrain audit started");
        AsyncComputeTaskPool::get()
            .spawn(async move {
                let result = run_audit(seed);
                if sender.send(result).is_err() {
                    tracing::error!(
                        target: "worldtools::audit",
                        "terrain audit result channel disconnected"
                    );
                }
            })
            .detach();
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
pub fn receive_audits(
    mut runtime: ResMut<AuditRuntime>,
    directory: Res<DiagnosticDirectory>,
    mut analysis: ResMut<AnalysisStatus>,
) {
    match runtime.receiver.try_recv() {
        Ok(Ok(report)) => {
            runtime.running = false;
            analysis.report_name = Some("Native terrain audit".to_owned());
            analysis.issues.clear();
            if !report.passed {
                analysis.issues.push(AnalysisIssue {
                    severity: AnalysisSeverity::Error,
                    label: "Seam, LOD, determinism, or finite-value audit failed".to_owned(),
                    location: None,
                });
            }
            match write_json_atomic(directory.path(), "terrain-audit", &report) {
                Ok(path) => tracing::info!(
                    target: "worldtools::audit",
                    passed = report.passed,
                    deterministic = report.deterministic,
                    duration_ms = report.duration_ms,
                    samples = report.terrain.elevation_vertex_samples,
                    non_finite = report.terrain.non_finite_elevation_vertex_samples,
                    seam_max_m = report.seam.maximum_absolute_error_m,
                    lod_max_m = report.lod.maximum_absolute_error_m,
                    land_fraction = report.terrain.land_vertex_weighted_fraction,
                    path = %path.display(),
                    "terrain audit completed"
                ),
                Err(error) => tracing::error!(
                    target: "worldtools::audit",
                    %error,
                    "terrain audit completed but its report could not be written"
                ),
            }
        }
        Ok(Err(error)) => {
            runtime.running = false;
            analysis.report_name = Some("Native terrain audit".to_owned());
            analysis.issues = vec![AnalysisIssue {
                severity: AnalysisSeverity::Error,
                label: error.clone(),
                location: None,
            }];
            tracing::error!(target: "worldtools::audit", %error, "terrain audit failed");
        }
        Err(TryRecvError::Empty) => {}
        Err(TryRecvError::Disconnected) => {
            runtime.running = false;
            tracing::error!(
                target: "worldtools::audit",
                "terrain audit worker channel disconnected"
            );
        }
    }
}

fn run_audit(seed: u64) -> AuditResult {
    let started = Instant::now();
    let settings = TerrainSettings::default();
    let generator = TerrainGenerator::new(WorldSeed(seed), settings);
    let root_tiles = CubeFace::ALL.map(|face| generator.generate(TileId::root(face)));
    let audits = root_tiles
        .iter()
        .map(|tile| {
            audit_terrain_at_sea_level(tile, settings.planet_radius_m, settings.sea_level_m)
        })
        .collect::<Vec<_>>();
    let terrain = aggregate_terrain(&audits)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "terrain aggregate was empty".to_owned())?;

    let positive_x = root_tile(&root_tiles, CubeFace::PositiveX)?;
    let negative_z = root_tile(&root_tiles, CubeFace::NegativeZ)?;
    let seam = audit_tile_seam(
        positive_x,
        TileEdge::East,
        negative_z,
        TileEdge::West,
        false,
    )
    .ok_or_else(|| "known cube-face seam was not geometrically aligned".to_owned())?;

    let parent = root_tile(&root_tiles, CubeFace::PositiveY)?;
    let child_id = parent.id.children().ok_or_else(|| {
        "root tile unexpectedly had no child coordinates for LOD audit".to_owned()
    })?[2];
    let child = generator.generate(child_id);
    let lod = audit_child_consistency(parent, &child)
        .ok_or_else(|| "parent-child LOD relationship was rejected".to_owned())?;
    let deterministic = positive_x.elevation_m()
        == generator
            .generate(TileId::root(CubeFace::PositiveX))
            .elevation_m();
    let passed = deterministic
        && terrain.non_finite_elevation_vertex_samples == 0
        && seam.maximum_absolute_error_m.to_bits() == 0.0_f32.to_bits()
        && lod.maximum_absolute_error_m.to_bits() == 0.0_f32.to_bits();

    Ok(TerrainAuditReport {
        schema: "worldtools.terrain-audit.v1",
        captured_unix_ms: timestamp_millis(),
        seed,
        duration_ms: started.elapsed().as_secs_f64() * 1_000.0,
        passed,
        deterministic,
        terrain,
        seam,
        lod,
    })
}

fn root_tile(
    tiles: &[worldtools_world::TerrainTile],
    face: CubeFace,
) -> Result<&worldtools_world::TerrainTile, String> {
    tiles
        .iter()
        .find(|tile| tile.id.face == face)
        .ok_or_else(|| format!("missing root tile for {face:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_checks_determinism_seams_and_lod() {
        let report = run_audit(7).unwrap();
        assert!(report.passed);
        assert!(report.deterministic);
        assert_eq!(report.terrain.tile_count, CubeFace::ALL.len());
        assert_eq!(report.seam.maximum_absolute_error_m.to_bits(), 0);
        assert_eq!(report.lod.maximum_absolute_error_m.to_bits(), 0);
    }
}
