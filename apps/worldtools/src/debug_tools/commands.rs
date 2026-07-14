use bevy::prelude::*;
use worldtools_render::{MapTileStreamer, RenderDebugSettings};
use worldtools_ui::{DebugCommand, DebugEventLog};

use super::{audit::AuditRequest, snapshot::SnapshotRequest};

pub fn handle_debug_commands(
    mut commands: MessageReader<DebugCommand>,
    mut render_debug: ResMut<RenderDebugSettings>,
    mut streamer: ResMut<MapTileStreamer>,
    mut event_log: ResMut<DebugEventLog>,
    mut snapshots: MessageWriter<SnapshotRequest>,
    mut audits: MessageWriter<AuditRequest>,
) {
    for command in commands.read() {
        match command {
            DebugCommand::SetRenderOptions(options) => {
                render_debug.tile_borders = options.tile_borders;
                render_debug.lod_tint = options.lod_tint;
                render_debug.residency_tint = options.fallback_tint;
                render_debug.trace_streaming = options.trace_streaming;
                tracing::info!(
                    target: "worldtools::debug",
                    tile_borders = options.tile_borders,
                    lod_tint = options.lod_tint,
                    residency_tint = options.fallback_tint,
                    trace_streaming = options.trace_streaming,
                    "render diagnostics changed"
                );
            }
            DebugCommand::SetStreamingFrozen(frozen) => {
                render_debug.freeze_streaming = *frozen;
                tracing::info!(
                    target: "worldtools::debug",
                    frozen,
                    "tile streaming freeze changed"
                );
            }
            DebugCommand::FlushTileCache => {
                let invalidated = streamer.invalidate_resident();
                tracing::info!(
                    target: "worldtools::debug",
                    invalidated,
                    "resident tile cache flushed"
                );
            }
            DebugCommand::CaptureSnapshot => {
                snapshots.write(SnapshotRequest);
            }
            DebugCommand::RunTerrainAudit => {
                audits.write(AuditRequest);
            }
            DebugCommand::ClearEvents => event_log.clear(),
        }
    }
}
