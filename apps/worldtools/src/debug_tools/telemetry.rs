use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameCount, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
    window::PrimaryWindow,
};
use worldtools_render::{MapView, MapViewport as RenderViewport, TileRenderStats, TileStreamStats};
use worldtools_ui::{
    DebugEvent, DebugEventLevel, DebugEventLog, DebugTelemetry, FrameDiagnostics, MapReadout,
    MapViewport as UiViewport, RenderDiagnostics, StreamingDiagnostics, ViewportDiagnostics,
};

use crate::diagnostics::{DiagnosticEventReceiver, DiagnosticLevel};

const MAX_TRACE_EVENTS_PER_FRAME: usize = 256;

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
pub fn drain_trace_events(
    receiver: Res<DiagnosticEventReceiver>,
    mut events: ResMut<DebugEventLog>,
) {
    events.dropped_events = events
        .dropped_events
        .saturating_add(receiver.take_dropped_events());

    for event in receiver.try_iter().take(MAX_TRACE_EVENTS_PER_FRAME) {
        let mut message = event.message;
        if !event.fields.is_empty() {
            message.push_str("  {");
            message.push_str(&event.fields);
            message.push('}');
        }
        if event.thread != "main" {
            message.push_str("  [");
            message.push_str(&event.thread);
            message.push(']');
        }
        events.push(DebugEvent::new(
            event.elapsed_seconds,
            event_level(event.level),
            event.target,
            message,
        ));
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::too_many_arguments,
    clippy::needless_pass_by_value
)] // Diagnostic samples are narrowed only for compact display telemetry.
pub fn sync_telemetry(
    diagnostics: Res<DiagnosticsStore>,
    frame_count: Res<FrameCount>,
    stream: Res<TileStreamStats>,
    renderer: Res<TileRenderStats>,
    view: Res<MapView>,
    render_viewport: Res<RenderViewport>,
    ui_viewport: Res<UiViewport>,
    readout: Res<MapReadout>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut telemetry: ResMut<DebugTelemetry>,
) {
    telemetry.frame = Some(FrameDiagnostics {
        frames_per_second: diagnostic_value(&diagnostics, &FrameTimeDiagnosticsPlugin::FPS)
            .unwrap_or_default() as f32,
        frame_time_ms: diagnostic_value(&diagnostics, &FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .unwrap_or_default() as f32,
        frame_number: u64::from(frame_count.0),
        entity_count: diagnostic_value(&diagnostics, &EntityCountDiagnosticsPlugin::ENTITY_COUNT)
            .unwrap_or_default() as u64,
        process_cpu_percent: diagnostic_value(
            &diagnostics,
            &SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE,
        )
        .map(|value| value as f32),
        process_memory_gib: diagnostic_value(
            &diagnostics,
            &SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE,
        )
        .map(|value| value as f32),
    });

    telemetry.streaming = Some(StreamingDiagnostics {
        level: stream.level,
        visible_tiles: stream.visible,
        resident_visible_tiles: stream.resident_visible,
        resident_total_tiles: stream.resident_total,
        in_flight_tiles: stream.in_flight,
        completed_jobs: stream.completed,
        discarded_jobs: stream.discarded,
        requested_jobs: stream.requested,
        invalidated_tiles: stream.invalidated,
        last_generation_ms: stream.last_generation_ms,
        max_generation_ms: stream.max_generation_ms,
        resident_capacity: stream.resident_capacity,
        max_in_flight: stream.max_in_flight,
        ready_results: stream.ready_results,
        edit_count: stream.edit_count,
    });

    telemetry.renderer = Some(RenderDiagnostics {
        rendered_tiles: renderer.rendered,
        exact_tiles: renderer.exact,
        fallback_tiles: renderer.fallback,
        stale_tiles: renderer.stale,
        missing_tiles: renderer.missing,
        gpu_resident_tiles: renderer.gpu_resident,
    });

    let window_logical = windows.single().ok().map_or(ui_viewport.logical, |window| {
        ui_viewport.window_logical(window.scale_factor())
    });
    let render_size = render_viewport.max - render_viewport.min;
    let logical_size = if render_size.min_element() > 1.0 {
        render_size.to_array()
    } else {
        [window_logical.width(), window_logical.height()]
    };
    telemetry.viewport = Some(ViewportDiagnostics {
        center_degrees: [
            f64::from(view.center.x) * 360.0 - 180.0,
            90.0 - f64::from(view.center.y) * 180.0,
        ],
        vertical_span_degrees: f64::from(view.vertical_span) * 180.0,
        logical_size,
        physical_size: [ui_viewport.physical.width(), ui_viewport.physical.height()],
        pixels_per_point: ui_viewport.pixels_per_point,
        lod: readout.lod,
        meters_per_pixel: readout.meters_per_pixel,
    });
}

fn diagnostic_value(
    diagnostics: &DiagnosticsStore,
    path: &bevy::diagnostic::DiagnosticPath,
) -> Option<f64> {
    diagnostics
        .get(path)
        .and_then(|diagnostic| diagnostic.smoothed().or_else(|| diagnostic.value()))
}

const fn event_level(level: DiagnosticLevel) -> DebugEventLevel {
    match level {
        DiagnosticLevel::Trace => DebugEventLevel::Trace,
        DiagnosticLevel::Debug => DebugEventLevel::Debug,
        DiagnosticLevel::Info => DebugEventLevel::Information,
        DiagnosticLevel::Warn => DebugEventLevel::Warning,
        DiagnosticLevel::Error => DebugEventLevel::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracing_levels_keep_their_severity() {
        assert_eq!(event_level(DiagnosticLevel::Warn), DebugEventLevel::Warning);
        assert_eq!(event_level(DiagnosticLevel::Error), DebugEventLevel::Error);
    }
}
