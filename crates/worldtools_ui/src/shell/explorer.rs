use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Vec2};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    DocumentStatus, EditorCommand, EditorUiState, LayerCapabilities, WorldLayer,
    style::{self, BG_PANEL, EXPLORER_WIDTH, TEXT_MUTED},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    document: &DocumentStatus,
    capabilities: &LayerCapabilities,
    commands: &mut MessageWriter<EditorCommand>,
) {
    egui::Panel::left("worldtools_explorer")
        .default_size(EXPLORER_WIDTH)
        .resizable(true)
        .size_range(220.0..=360.0)
        .frame(style::panel_frame(BG_PANEL))
        .show(root, |ui| {
            ui.label(
                egui::RichText::new("WORLD EXPLORER")
                    .size(10.0)
                    .strong()
                    .color(TEXT_MUTED),
            );
            ui.add_space(5.0);

            egui::CollapsingHeader::new(format!("{}  {}", icons::PLANET.as_str(), document.name))
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("seed {}", document.seed))
                            .small()
                            .color(TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  Generator graph",
                            icons::TREE_STRUCTURE.as_str()
                        ))
                        .color(TEXT_MUTED),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  Edit journal",
                            icons::NOTE_PENCIL.as_str()
                        ))
                        .color(TEXT_MUTED),
                    );
                });

            widgets::section_header(ui, "Layers");
            for layer in WorldLayer::ALL {
                layer_row(ui, state, capabilities, layer, commands);
            }
        });
}

fn layer_row(
    ui: &mut egui::Ui,
    state: &mut EditorUiState,
    capabilities: &LayerCapabilities,
    layer: WorldLayer,
    commands: &mut MessageWriter<EditorCommand>,
) {
    let availability = capabilities.availability(layer);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
        let mut visible = state.layer_visible(layer);
        let eye = if visible {
            icons::EYE
        } else {
            icons::EYE_SLASH
        };
        if ui
            .add_enabled_ui(availability.is_available(), |ui| {
                ui.add_sized(
                    [24.0, 22.0],
                    egui::Button::new(style::icon_text(eye, 14.0, TEXT_MUTED)).frame(false),
                )
            })
            .inner
            .on_hover_text(if visible { "Hide layer" } else { "Show layer" })
            .clicked()
        {
            visible = !visible;
            state.set_layer_visible(layer, visible);
            commands.write(EditorCommand::SetLayerVisibility { layer, visible });
        }

        let selected = state.active_layer == layer;
        let response = ui
            .add_enabled_ui(availability.is_available(), |ui| {
                ui.add_sized(
                    [ui.available_width(), 22.0],
                    egui::Button::new(format!("{}  {}", layer_icon(layer).as_str(), layer.label()))
                        .selected(selected),
                )
            })
            .inner;
        let response = if let Some(reason) = availability.reason() {
            response.on_hover_text(format!("Unavailable: {reason}"))
        } else {
            response
        };
        if response.clicked() && !selected {
            state.active_layer = layer;
            commands.write(EditorCommand::SelectLayer(layer));
        }
    });
}

fn layer_icon(layer: WorldLayer) -> Icon {
    match layer {
        WorldLayer::Elevation => icons::MOUNTAINS,
        WorldLayer::Tectonics => icons::CIRCLES_FOUR,
        WorldLayer::Hydrology => icons::WAVES,
        WorldLayer::Climate => icons::WIND,
        WorldLayer::Soil => icons::SHOVEL,
        WorldLayer::Vegetation => icons::TREE_EVERGREEN,
        WorldLayer::Geology => icons::DIAMOND,
        WorldLayer::Resources => icons::HAMMER,
    }
}
