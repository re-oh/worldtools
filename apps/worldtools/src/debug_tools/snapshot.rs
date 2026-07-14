use bevy::{diagnostic::SystemInfo, prelude::*};
use serde_json::{Value, json};
use worldtools_render::{
    MapTileId, MapTileStreamer, MapView, RenderDebugSettings, TileRenderStats, TileStreamStats,
};
use worldtools_ui::{
    DebugEventLog, DebugTelemetry, DebugUiState, DocumentStatus, EditorUiState, GenerationStatus,
    LayerCapabilities, WorldLayer,
};

use crate::diagnostics::DiagnosticDirectory;

use super::io::{timestamp_millis, write_json_atomic};

#[derive(Clone, Copy, Debug, Message)]
pub struct SnapshotRequest;

#[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
pub fn capture_snapshots(
    mut requests: MessageReader<SnapshotRequest>,
    directory: Res<DiagnosticDirectory>,
    document: Res<DocumentStatus>,
    editor: Res<EditorUiState>,
    generation: Res<GenerationStatus>,
    debug_ui: Res<DebugUiState>,
    telemetry: Res<DebugTelemetry>,
    events: Res<DebugEventLog>,
    capabilities: Res<LayerCapabilities>,
    render_debug: Res<RenderDebugSettings>,
    stream_stats: Res<TileStreamStats>,
    render_stats: Res<TileRenderStats>,
    streamer: Res<MapTileStreamer>,
    view: Res<MapView>,
    system: Option<Res<SystemInfo>>,
) {
    for _ in requests.read() {
        let context = SnapshotContext {
            document: &document,
            editor: &editor,
            generation: &generation,
            debug_ui: &debug_ui,
            telemetry: &telemetry,
            events: &events,
            capabilities: &capabilities,
            render_debug: &render_debug,
            stream_stats: &stream_stats,
            render_stats: &render_stats,
            streamer: &streamer,
            view: &view,
            system: system.as_deref(),
        };
        let snapshot = snapshot_value(&context);
        match write_json_atomic(directory.path(), "snapshot", &snapshot) {
            Ok(path) => tracing::info!(
                target: "worldtools::debug",
                path = %path.display(),
                "diagnostic snapshot captured"
            ),
            Err(error) => tracing::error!(
                target: "worldtools::debug",
                %error,
                directory = %directory.path().display(),
                "diagnostic snapshot failed"
            ),
        }
    }
}

struct SnapshotContext<'a> {
    document: &'a DocumentStatus,
    editor: &'a EditorUiState,
    generation: &'a GenerationStatus,
    debug_ui: &'a DebugUiState,
    telemetry: &'a DebugTelemetry,
    events: &'a DebugEventLog,
    capabilities: &'a LayerCapabilities,
    render_debug: &'a RenderDebugSettings,
    stream_stats: &'a TileStreamStats,
    render_stats: &'a TileRenderStats,
    streamer: &'a MapTileStreamer,
    view: &'a MapView,
    system: Option<&'a SystemInfo>,
}

fn snapshot_value(context: &SnapshotContext<'_>) -> Value {
    let (resident, in_flight) = tile_values(context.streamer);
    let layers = layer_values(context);
    let recent_events = recent_event_values(context.events);

    json!({
        "schema": "worldtools.diagnostic-snapshot.v2",
        "captured_unix_ms": timestamp_millis(),
        "application": {
            "version": env!("CARGO_PKG_VERSION"),
            "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
            "working_directory": std::env::current_dir().ok(),
        },
        "system": context.system.map(|system| json!({
            "os": system.os,
            "kernel": system.kernel,
            "cpu": system.cpu,
            "core_count": system.core_count,
            "memory": system.memory,
        })),
        "document": {
            "name": context.document.name,
            "seed": context.document.seed,
            "active_tool": context.editor.active_tool.label(),
            "active_layer": context.editor.active_layer.label(),
        },
        "generation": {
            "activity": format!("{:?}", context.generation.activity),
        },
        "simulation": simulation_value(context.streamer),
        "view": {
            "center": context.view.center.to_array(),
            "vertical_span": context.view.vertical_span,
        },
        "debug": {
            "window_visible": context.debug_ui.visible,
            "selected_tab": context.debug_ui.selected_tab.label(),
            "tile_borders": context.render_debug.tile_borders,
            "lod_tint": context.render_debug.lod_tint,
            "residency_tint": context.render_debug.residency_tint,
            "trace_streaming": context.render_debug.trace_streaming,
            "freeze_streaming": context.render_debug.freeze_streaming,
        },
        "telemetry": telemetry_value(context.telemetry),
        "stream_stats": stream_stats_value(context.stream_stats),
        "render_stats": render_stats_value(context.render_stats),
        "layers": layers,
        "tiles": {
            "resident": resident,
            "in_flight": in_flight,
        },
        "events": {
            "buffered": context.events.len(),
            "dropped": context.events.dropped_events,
            "recent": recent_events,
        },
    })
}

fn tile_values(streamer: &MapTileStreamer) -> (Vec<Value>, Vec<Value>) {
    let values = |ids: Vec<MapTileId>| {
        ids.into_iter()
            .map(|id| tile_value(id, streamer.tile_revision(id)))
            .collect()
    };
    (
        values(streamer.resident_tile_ids()),
        values(streamer.in_flight_tile_ids()),
    )
}

fn layer_values(context: &SnapshotContext<'_>) -> Vec<Value> {
    WorldLayer::ALL
        .into_iter()
        .map(|layer| {
            let availability = context.capabilities.availability(layer);
            json!({
                "name": layer.label(),
                "available": availability.is_available(),
                "reason": availability.reason(),
                "selected": context.editor.active_layer == layer,
            })
        })
        .collect()
}

fn recent_event_values(events: &DebugEventLog) -> Vec<Value> {
    let mut values = events
        .iter()
        .rev()
        .take(200)
        .map(|event| {
            json!({
                "elapsed_seconds": event.elapsed_seconds,
                "level": event.level.label(),
                "target": event.target,
                "message": event.message,
            })
        })
        .collect::<Vec<_>>();
    values.reverse();
    values
}

fn simulation_value(streamer: &MapTileStreamer) -> Value {
    let world = streamer.snapshot();
    let settings = world.simulation_settings();
    json!({
        "active_layer": streamer.active_layer().label(),
        "revision": world.revision(),
        "fingerprint": fingerprint_hex(world.fingerprint()),
        "atlas_width": settings.atlas_width,
        "atlas_height": settings.atlas_height,
        "climate_width": settings.climate_width,
        "climate_height": settings.climate_height,
        "plate_count": settings.plate_count,
        "hotspot_count": settings.hotspot_count,
        "geological_age_myr": settings.geological_age_myr,
        "erosion_iterations": settings.erosion_iterations,
        "moisture_iterations": settings.moisture_iterations,
        "glacial_iterations": settings.glacial_iterations,
    })
}

fn fingerprint_hex(fingerprint: [u8; 32]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(fingerprint.len() * 2);
    for byte in fingerprint {
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    encoded
}

fn tile_value(id: MapTileId, revision: u64) -> Value {
    json!({ "level": id.level, "x": id.x, "y": id.y, "revision": revision })
}

fn stream_stats_value(stats: &TileStreamStats) -> Value {
    json!({
        "level": stats.level,
        "visible": stats.visible,
        "resident_visible": stats.resident_visible,
        "resident_total": stats.resident_total,
        "resident_capacity": stats.resident_capacity,
        "in_flight": stats.in_flight,
        "max_in_flight": stats.max_in_flight,
        "ready_results": stats.ready_results,
        "requested": stats.requested,
        "completed": stats.completed,
        "discarded": stats.discarded,
        "invalidated": stats.invalidated,
        "last_generation_ms": stats.last_generation_ms,
        "max_generation_ms": stats.max_generation_ms,
        "world_epoch": stats.world_epoch,
    })
}

fn render_stats_value(stats: &TileRenderStats) -> Value {
    json!({
        "rendered": stats.rendered,
        "exact": stats.exact,
        "fallback": stats.fallback,
        "stale": stats.stale,
        "missing": stats.missing,
        "gpu_resident": stats.gpu_resident,
    })
}

fn telemetry_value(telemetry: &DebugTelemetry) -> Value {
    json!({
        "frame": telemetry.frame.as_ref().map(|frame| json!({
            "fps": frame.frames_per_second,
            "frame_time_ms": frame.frame_time_ms,
            "frame_number": frame.frame_number,
            "entity_count": frame.entity_count,
            "process_cpu_percent": frame.process_cpu_percent,
            "process_memory_gib": frame.process_memory_gib,
        })),
        "streaming": telemetry.streaming.as_ref().map(|stream| json!({
            "world_epoch": stream.world_epoch,
            "level": stream.level,
            "visible_tiles": stream.visible_tiles,
            "resident_visible_tiles": stream.resident_visible_tiles,
            "resident_total_tiles": stream.resident_total_tiles,
            "in_flight_tiles": stream.in_flight_tiles,
            "completed_jobs": stream.completed_jobs,
            "discarded_jobs": stream.discarded_jobs,
            "requested_jobs": stream.requested_jobs,
            "invalidated_tiles": stream.invalidated_tiles,
            "last_generation_ms": stream.last_generation_ms,
            "max_generation_ms": stream.max_generation_ms,
            "ready_results": stream.ready_results,
        })),
        "viewport": telemetry.viewport.as_ref().map(|viewport| json!({
            "center_degrees": viewport.center_degrees,
            "vertical_span_degrees": viewport.vertical_span_degrees,
            "logical_size": viewport.logical_size,
            "physical_size": viewport.physical_size,
            "pixels_per_point": viewport.pixels_per_point,
            "lod": viewport.lod,
            "meters_per_pixel": viewport.meters_per_pixel,
        })),
        "renderer": telemetry.renderer.as_ref().map(|renderer| json!({
            "rendered_tiles": renderer.rendered_tiles,
            "exact_tiles": renderer.exact_tiles,
            "fallback_tiles": renderer.fallback_tiles,
            "stale_tiles": renderer.stale_tiles,
            "missing_tiles": renderer.missing_tiles,
            "gpu_resident_tiles": renderer.gpu_resident_tiles,
        })),
    })
}
