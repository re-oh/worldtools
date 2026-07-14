use bevy::prelude::MessageWriter;
use bevy_egui::egui;
use egui_phosphor_icons::icons;

use crate::{
    ActiveTool, BrushFalloff, BrushOperation, EditorCommand, EditorUiState, GenerationActivity,
    GenerationScope, GenerationStatus, LayerCapabilities,
    style::{self, BG_PANEL, INSPECTOR_WIDTH, TEXT_MUTED, WARNING},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    generation: &GenerationStatus,
    capabilities: &LayerCapabilities,
    commands: &mut MessageWriter<EditorCommand>,
) {
    egui::Panel::right("worldtools_inspector")
        .default_size(INSPECTOR_WIDTH)
        .resizable(true)
        .size_range(280.0..=420.0)
        .frame(style::panel_frame(BG_PANEL))
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("INSPECTOR")
                        .size(10.0)
                        .strong()
                        .color(TEXT_MUTED),
                );
                ui.separator();
                ui.label(state.active_tool.label());
            });

            widgets::section_header(ui, "Layer");
            widgets::property_row(ui, "Active", |ui| {
                egui::ComboBox::from_id_salt("inspector_active_layer")
                    .selected_text(state.active_layer.label())
                    .show_ui(ui, |ui| {
                        for layer in crate::WorldLayer::ALL {
                            let availability = capabilities.availability(layer);
                            let changed = ui
                                .add_enabled_ui(availability.is_available(), |ui| {
                                    ui.selectable_value(
                                        &mut state.active_layer,
                                        layer,
                                        layer.label(),
                                    )
                                })
                                .inner
                                .on_hover_text(
                                    availability.reason().unwrap_or("Native layer available"),
                                )
                                .changed();
                            if changed {
                                commands.write(EditorCommand::SelectLayer(layer));
                            }
                        }
                    });
            });

            let mut opacity = state.layer_opacity(state.active_layer);
            widgets::property_row(ui, "Opacity", |ui| {
                if ui
                    .add(egui::Slider::new(&mut opacity, 0.0..=1.0).show_value(true))
                    .changed()
                {
                    state.set_layer_opacity(state.active_layer, opacity);
                    commands.write(EditorCommand::SetLayerOpacity {
                        layer: state.active_layer,
                        opacity,
                    });
                }
            });

            if state.active_tool.uses_brush() {
                brush_controls(ui, state, commands);
            } else if state.active_tool == ActiveTool::Inspect {
                widgets::section_header(ui, "Sample");
                ui.label(
                    egui::RichText::new("No map sample selected")
                        .color(TEXT_MUTED)
                        .italics(),
                );
            }

            generation_controls(ui, state, generation, commands);
        });
}

fn brush_controls(
    ui: &mut egui::Ui,
    state: &mut EditorUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
    widgets::section_header(ui, "Brush");
    let before = state.brush;

    if state.active_tool == ActiveTool::Sculpt {
        widgets::property_row(ui, "Operation", |ui| {
            egui::ComboBox::from_id_salt("brush_operation")
                .selected_text(state.brush.operation.label())
                .show_ui(ui, |ui| {
                    for operation in BrushOperation::ALL {
                        ui.selectable_value(
                            &mut state.brush.operation,
                            operation,
                            operation.label(),
                        );
                    }
                });
        });
    }

    widgets::property_row(ui, "Radius", |ui| {
        ui.add(
            egui::DragValue::new(&mut state.brush.radius_m)
                .range(1.0..=2_000_000.0)
                .speed(100.0)
                .suffix(" m"),
        );
    });
    widgets::property_row(ui, "Strength", |ui| {
        ui.add(egui::Slider::new(&mut state.brush.strength, 0.0..=1.0));
    });
    widgets::property_row(ui, "Falloff", |ui| {
        egui::ComboBox::from_id_salt("brush_falloff")
            .selected_text(state.brush.falloff.label())
            .show_ui(ui, |ui| {
                for falloff in BrushFalloff::ALL {
                    ui.selectable_value(&mut state.brush.falloff, falloff, falloff.label());
                }
            });
    });
    widgets::property_row(ui, "Spacing", |ui| {
        ui.add(egui::Slider::new(&mut state.brush.spacing, 0.02..=1.0));
    });

    state.brush.sanitize();
    if state.brush != before {
        commands.write(EditorCommand::UpdateBrush(state.brush));
    }

    ui.add_space(3.0);
    if ui
        .button(format!(
            "{} Clear {} edits",
            icons::ERASER.as_str(),
            state.active_layer.label()
        ))
        .clicked()
    {
        commands.write(EditorCommand::ClearLayerEdits(state.active_layer));
    }
}

fn generation_controls(
    ui: &mut egui::Ui,
    state: &mut EditorUiState,
    generation: &GenerationStatus,
    commands: &mut MessageWriter<EditorCommand>,
) {
    widgets::section_header(ui, "Generation");
    widgets::property_row(ui, "Scope", |ui| {
        egui::ComboBox::from_id_salt("generation_scope")
            .selected_text(state.generation_scope.label())
            .show_ui(ui, |ui| {
                for scope in GenerationScope::ALL {
                    ui.selectable_value(&mut state.generation_scope, scope, scope.label());
                }
            });
    });

    let mut auto_rebuild = state.auto_rebuild;
    if ui
        .checkbox(&mut auto_rebuild, "Auto rebuild affected tiles")
        .changed()
    {
        state.auto_rebuild = auto_rebuild;
        commands.write(EditorCommand::SetAutoRebuild(auto_rebuild));
    }

    if generation.dirty.is_empty() {
        ui.horizontal(|ui| {
            widgets::status_indicator(ui, style::VALID);
            ui.label(egui::RichText::new("Local pipeline is current").color(TEXT_MUTED));
        });
    } else {
        let stage = generation
            .dirty
            .from_stage
            .map_or_else(|| "pipeline".to_owned(), |stage| stage.to_string());
        ui.horizontal(|ui| {
            widgets::status_indicator(ui, WARNING);
            ui.label(
                egui::RichText::new(format!(
                    "{} tiles dirty · {stage} -> resources",
                    generation.dirty.tile_count
                ))
                .color(WARNING),
            );
        });
    }

    ui.horizontal(|ui| {
        let busy = matches!(generation.activity, GenerationActivity::Running { .. });
        if ui
            .add_enabled(
                !busy
                    && (!generation.dirty.is_empty()
                        || state.generation_scope != GenerationScope::Dirty),
                egui::Button::new(format!("{} Rebuild", icons::PLAY.as_str())),
            )
            .clicked()
        {
            commands.write(EditorCommand::Generate(state.generation_scope));
        }
        if !generation.dirty.is_empty()
            && ui
                .button(format!("{} Locate", icons::CROSSHAIR.as_str()))
                .clicked()
        {
            commands.write(EditorCommand::FocusDirtyRegion);
        }
    });
}
