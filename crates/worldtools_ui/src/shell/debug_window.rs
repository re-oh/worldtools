use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Color32};
use egui_phosphor_icons::icons;

use crate::{
    DebugCommand, DebugEventLevel, DebugEventLog, DebugTab, DebugTelemetry, DebugUiState,
    EditorUiState, GenerationActivity, GenerationStatus, LayerAvailability, LayerCapabilities,
    WorldLayer,
    style::{self, BG_PANEL, ERROR, TEXT_MUTED, VALID, WARNING},
    widgets,
};

#[allow(clippy::too_many_arguments)]
pub fn show(
    ctx: &egui::Context,
    state: &mut DebugUiState,
    telemetry: &DebugTelemetry,
    events: &DebugEventLog,
    capabilities: &LayerCapabilities,
    editor: &EditorUiState,
    generation: &GenerationStatus,
    commands: &mut MessageWriter<DebugCommand>,
) {
    if !state.visible {
        return;
    }

    let mut open = state.visible;
    egui::Window::new(format!("{} DIAGNOSTICS", icons::BUG.as_str()))
        .id(egui::Id::new("worldtools_debug_window"))
        .open(&mut open)
        .default_pos([52.0, 44.0])
        .default_size([540.0, 390.0])
        .min_size([440.0, 280.0])
        .resizable(true)
        .frame(style::panel_frame(BG_PANEL))
        .show(ctx, |ui| {
            tabs(ui, state);
            ui.separator();

            match state.selected_tab {
                DebugTab::Summary => summary(ui, telemetry, generation, commands),
                DebugTab::Streaming => streaming(ui, state, telemetry, commands),
                DebugTab::Viewport => viewport(ui, telemetry),
                DebugTab::Layers => layers(ui, capabilities, editor),
                DebugTab::Events => event_log(ui, state, events, commands),
            }
        });
    state.visible = open;
}

fn tabs(ui: &mut egui::Ui, state: &mut DebugUiState) {
    ui.horizontal(|ui| {
        for tab in DebugTab::ALL {
            ui.selectable_value(&mut state.selected_tab, tab, tab.label());
        }
    });
}

fn summary(
    ui: &mut egui::Ui,
    telemetry: &DebugTelemetry,
    generation: &GenerationStatus,
    commands: &mut MessageWriter<DebugCommand>,
) {
    egui::Grid::new("debug_summary_grid")
        .num_columns(4)
        .spacing([12.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            if let Some(frame) = &telemetry.frame {
                datum(ui, "FPS", format!("{:.1}", frame.frames_per_second));
                datum(ui, "FRAME", format!("{:.2} ms", frame.frame_time_ms));
                ui.end_row();
                datum(ui, "FRAME NO.", frame.frame_number.to_string());
                datum(ui, "ENTITIES", frame.entity_count.to_string());
                ui.end_row();
                datum(
                    ui,
                    "PROCESS CPU",
                    frame
                        .process_cpu_percent
                        .map_or_else(|| "waiting".to_owned(), |value| format!("{value:.1}%")),
                );
                datum(
                    ui,
                    "PROCESS RAM",
                    frame
                        .process_memory_gib
                        .map_or_else(|| "waiting".to_owned(), |value| format!("{value:.2} GiB")),
                );
                ui.end_row();
            } else {
                disconnected_row(ui, "Frame diagnostics");
            }

            if let Some(render) = &telemetry.renderer {
                datum(ui, "RENDERED", render.rendered_tiles.to_string());
                datum(ui, "GPU PAGES", render.gpu_resident_tiles.to_string());
                ui.end_row();
                datum(ui, "EXACT", render.exact_tiles.to_string());
                datum(
                    ui,
                    "DEGRADED",
                    (render.fallback_tiles + render.stale_tiles + render.missing_tiles).to_string(),
                );
                ui.end_row();
            } else {
                disconnected_row(ui, "Render diagnostics");
            }
        });

    widgets::section_header(ui, "Pipeline");
    let (label, color) = match &generation.activity {
        GenerationActivity::Idle => ("idle".to_owned(), VALID),
        GenerationActivity::Queued { jobs } => (format!("{jobs} jobs queued"), WARNING),
        GenerationActivity::Running {
            stage,
            completed,
            total,
        } => (format!("{stage}: {completed}/{total}"), WARNING),
        GenerationActivity::Failed { message } => (message.clone(), ERROR),
    };
    ui.horizontal(|ui| {
        widgets::status_indicator(ui, color);
        ui.label(label);
        ui.separator();
        ui.label(
            egui::RichText::new(format!("{} dirty tiles", generation.dirty.tile_count))
                .color(TEXT_MUTED),
        );
    });

    widgets::section_header(ui, "Capture");
    ui.monospace(".runtime/diagnostics");
    ui.horizontal(|ui| {
        if ui
            .button(format!("{} Snapshot", icons::CAMERA.as_str()))
            .on_hover_text("Write a diagnostic snapshot with current telemetry and world state")
            .clicked()
        {
            commands.write(DebugCommand::CaptureSnapshot);
        }
        if ui
            .button(format!("{} Terrain audit", icons::CHECKS.as_str()))
            .on_hover_text("Run deterministic seam, LOD, and terrain-distribution checks")
            .clicked()
        {
            commands.write(DebugCommand::RunTerrainAudit);
        }
    });
}

#[allow(clippy::too_many_lines)] // One cohesive diagnostics tab; extracting grid rows obscures the layout.
fn streaming(
    ui: &mut egui::Ui,
    state: &mut DebugUiState,
    telemetry: &DebugTelemetry,
    commands: &mut MessageWriter<DebugCommand>,
) {
    widgets::section_header(ui, "Tile stream");
    if let Some(stream) = &telemetry.streaming {
        egui::Grid::new("debug_stream_grid")
            .num_columns(4)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                datum(ui, "LOD", stream.level.to_string());
                datum(ui, "VISIBLE", stream.visible_tiles.to_string());
                ui.end_row();
                datum(
                    ui,
                    "RESIDENT VISIBLE",
                    stream.resident_visible_tiles.to_string(),
                );
                datum(
                    ui,
                    "RESIDENT TOTAL",
                    stream.resident_total_tiles.to_string(),
                );
                ui.end_row();
                datum(ui, "IN FLIGHT", stream.in_flight_tiles.to_string());
                datum(ui, "COMPLETED", stream.completed_jobs.to_string());
                ui.end_row();
                datum(ui, "DISCARDED", stream.discarded_jobs.to_string());
                datum(ui, "REQUESTED", stream.requested_jobs.to_string());
                ui.end_row();
                datum(ui, "INVALIDATED", stream.invalidated_tiles.to_string());
                datum(ui, "EDITS", stream.edit_count.to_string());
                ui.end_row();
                datum(
                    ui,
                    "LAST GENERATION",
                    format!("{:.2} ms", stream.last_generation_ms),
                );
                datum(
                    ui,
                    "MAX GENERATION",
                    format!("{:.2} ms", stream.max_generation_ms),
                );
                ui.end_row();
                datum(
                    ui,
                    "CACHE",
                    format!(
                        "{}/{}",
                        stream.resident_total_tiles, stream.resident_capacity
                    ),
                );
                datum(
                    ui,
                    "QUEUE",
                    format!(
                        "{} ready / {} max",
                        stream.ready_results, stream.max_in_flight
                    ),
                );
                ui.end_row();
            });
    } else {
        disconnected(ui, "Tile stream diagnostics are not connected");
    }

    widgets::section_header(ui, "Rendered pages");
    if let Some(render) = &telemetry.renderer {
        egui::Grid::new("debug_render_grid")
            .num_columns(4)
            .spacing([12.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                datum(ui, "EXACT", render.exact_tiles.to_string());
                datum(ui, "FALLBACK", render.fallback_tiles.to_string());
                ui.end_row();
                datum(ui, "STALE", render.stale_tiles.to_string());
                datum(ui, "MISSING", render.missing_tiles.to_string());
                ui.end_row();
                datum(ui, "GPU RESIDENT", render.gpu_resident_tiles.to_string());
            });
    } else {
        disconnected(ui, "Render diagnostics are not connected");
    }

    widgets::section_header(ui, "Controls");
    let before = state.render_options;
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut state.render_options.tile_borders, "Tile borders");
        ui.checkbox(&mut state.render_options.lod_tint, "LOD tint");
        ui.checkbox(&mut state.render_options.fallback_tint, "Residency tint");
        ui.checkbox(&mut state.render_options.trace_streaming, "Trace lifecycle");
    });
    if state.render_options != before {
        commands.write(DebugCommand::SetRenderOptions(state.render_options));
    }

    let mut frozen = state.freeze_streaming;
    if ui.checkbox(&mut frozen, "Freeze tile streaming").changed() {
        state.freeze_streaming = frozen;
        commands.write(DebugCommand::SetStreamingFrozen(frozen));
    }
    if ui
        .button(format!("{} Flush tile cache", icons::TRASH.as_str()))
        .clicked()
    {
        commands.write(DebugCommand::FlushTileCache);
    }
}

fn viewport(ui: &mut egui::Ui, telemetry: &DebugTelemetry) {
    widgets::section_header(ui, "Projection");
    let Some(viewport) = &telemetry.viewport else {
        disconnected(ui, "Viewport diagnostics are not connected");
        return;
    };

    egui::Grid::new("debug_viewport_grid")
        .num_columns(2)
        .spacing([14.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            property(
                ui,
                "Center longitude",
                format!("{:.6} deg", viewport.center_degrees[0]),
            );
            property(
                ui,
                "Center latitude",
                format!("{:.6} deg", viewport.center_degrees[1]),
            );
            property(
                ui,
                "Vertical span",
                format!("{:.6} deg", viewport.vertical_span_degrees),
            );
            property(
                ui,
                "Logical viewport",
                format!(
                    "{:.0} x {:.0}",
                    viewport.logical_size[0], viewport.logical_size[1]
                ),
            );
            property(
                ui,
                "Physical viewport",
                format!(
                    "{:.0} x {:.0}",
                    viewport.physical_size[0], viewport.physical_size[1]
                ),
            );
            property(
                ui,
                "Pixels per point",
                format!("{:.3}", viewport.pixels_per_point),
            );
            property(ui, "Selected LOD", viewport.lod.to_string());
            property(
                ui,
                "Ground resolution",
                format_distance(viewport.meters_per_pixel),
            );
        });
}

fn layers(ui: &mut egui::Ui, capabilities: &LayerCapabilities, editor: &EditorUiState) {
    widgets::section_header(ui, "Native layer contract");
    egui::Grid::new("debug_layers_grid")
        .num_columns(4)
        .spacing([10.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.strong("LAYER");
            ui.strong("STATE");
            ui.strong("VISIBLE");
            ui.strong("OPACITY");
            ui.end_row();
            for layer in WorldLayer::ALL {
                ui.label(layer.label());
                match capabilities.availability(layer) {
                    LayerAvailability::Available => {
                        ui.colored_label(VALID, "AVAILABLE");
                    }
                    LayerAvailability::Unavailable(reason) => {
                        ui.colored_label(WARNING, "UNAVAILABLE")
                            .on_hover_text(reason);
                    }
                }
                ui.label(if editor.layer_visible(layer) {
                    "yes"
                } else {
                    "no"
                });
                ui.label(format!("{:.0}%", editor.layer_opacity(layer) * 100.0));
                ui.end_row();
            }
        });
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(
            "Unavailable controls are disabled in the explorer and inspector; they are not no-ops.",
        )
        .color(TEXT_MUTED),
    );
}

fn event_log(
    ui: &mut egui::Ui,
    state: &mut DebugUiState,
    events: &DebugEventLog,
    commands: &mut MessageWriter<DebugCommand>,
) {
    ui.horizontal(|ui| {
        ui.label("Filter");
        ui.add(
            egui::TextEdit::singleline(&mut state.event_filter)
                .desired_width(220.0)
                .hint_text("target or message"),
        );
        ui.checkbox(&mut state.follow_events, "Follow");
        if ui
            .button(format!("{} Clear", icons::TRASH.as_str()))
            .clicked()
        {
            commands.write(DebugCommand::ClearEvents);
        }
    });
    ui.label(
        egui::RichText::new(format!(
            "{} buffered / {} dropped",
            events.len(),
            events.dropped_events
        ))
        .small()
        .color(TEXT_MUTED),
    );

    let filter = state.event_filter.to_lowercase();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(state.follow_events)
        .show(ui, |ui| {
            for event in events.iter() {
                if !filter.is_empty()
                    && !event.target.to_lowercase().contains(&filter)
                    && !event.message.to_lowercase().contains(&filter)
                {
                    continue;
                }
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{:8.3}", event.elapsed_seconds))
                            .color(TEXT_MUTED),
                    );
                    ui.colored_label(level_color(event.level), event.level.label());
                    ui.label(egui::RichText::new(&event.target).color(style::ACCENT));
                    ui.label(&event.message);
                });
            }
            if events.is_empty() {
                disconnected(ui, "No diagnostic events captured");
            }
        });
}

fn datum(ui: &mut egui::Ui, label: &str, value: String) {
    ui.label(egui::RichText::new(label).small().color(TEXT_MUTED));
    ui.monospace(value);
}

fn property(ui: &mut egui::Ui, label: &str, value: String) {
    ui.label(egui::RichText::new(label).color(TEXT_MUTED));
    ui.monospace(value);
    ui.end_row();
}

fn disconnected_row(ui: &mut egui::Ui, label: &str) {
    ui.label(egui::RichText::new(label).color(TEXT_MUTED));
    ui.colored_label(WARNING, "NOT CONNECTED");
    ui.end_row();
}

fn disconnected(ui: &mut egui::Ui, message: &str) {
    ui.horizontal(|ui| {
        widgets::status_indicator(ui, WARNING);
        ui.label(egui::RichText::new(message).color(TEXT_MUTED));
    });
}

fn level_color(level: DebugEventLevel) -> Color32 {
    match level {
        DebugEventLevel::Trace | DebugEventLevel::Debug => TEXT_MUTED,
        DebugEventLevel::Information => style::TEXT,
        DebugEventLevel::Warning => WARNING,
        DebugEventLevel::Error => ERROR,
    }
}

fn format_distance(meters: f64) -> String {
    if meters >= 1_000.0 {
        format!("{:.3} km/px", meters / 1_000.0)
    } else {
        format!("{meters:.3} m/px")
    }
}
