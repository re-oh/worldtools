use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Vec2};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    DocumentStatus, EditorUiState, GenerationStatus, LayerCapabilities, RegenerateWorld,
    WorldGenerationDraft, WorldLayer,
    style::{self, BG_PANEL, EXPLORER_WIDTH, TEXT_MUTED},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    document: &DocumentStatus,
    draft: &mut WorldGenerationDraft,
    generation: &GenerationStatus,
    capabilities: &LayerCapabilities,
    regeneration: &mut MessageWriter<RegenerateWorld>,
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
                        egui::RichText::new(format!("active seed {}", document.seed))
                            .small()
                            .color(TEXT_MUTED),
                    );
                    let mut submit = false;
                    widgets::property_row(ui, "Seed", |ui| {
                        let response = ui.add(
                            egui::DragValue::new(&mut draft.seed)
                                .speed(1.0)
                                .update_while_editing(false),
                        );
                        submit = response.lost_focus()
                            && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    });
                    let busy = generation.is_running();
                    submit |= ui
                        .add_enabled(
                            !busy,
                            egui::Button::new(format!(
                                "{} REGENERATE WORLD",
                                icons::ARROWS_CLOCKWISE.as_str()
                            )),
                        )
                        .on_hover_text(if busy {
                            "World history generation is running"
                        } else {
                            "Rebuild every world-data layer from this seed"
                        })
                        .clicked();
                    if submit && !busy {
                        regeneration.write(RegenerateWorld { seed: draft.seed });
                    }
                });

            widgets::section_header(ui, "Data");
            for layer in WorldLayer::ALL
                .into_iter()
                .filter(|layer| capabilities.availability(*layer).is_available())
            {
                layer_row(ui, state, capabilities, layer);
            }

            future_data(ui, capabilities);
        });
}

fn future_data(ui: &mut egui::Ui, capabilities: &LayerCapabilities) {
    let unavailable = WorldLayer::ALL
        .into_iter()
        .filter(|layer| !capabilities.availability(*layer).is_available())
        .collect::<Vec<_>>();
    if unavailable.is_empty() {
        return;
    }

    ui.add_space(4.0);
    egui::CollapsingHeader::new(
        egui::RichText::new(format!("Future data ({})", unavailable.len()))
            .small()
            .color(TEXT_MUTED),
    )
    .default_open(false)
    .show(ui, |ui| {
        for layer in unavailable {
            let reason = capabilities
                .availability(layer)
                .reason()
                .unwrap_or("unavailable");
            ui.horizontal(|ui| {
                ui.label(style::icon_text(layer_icon(layer), 13.0, TEXT_MUTED));
                ui.label(egui::RichText::new(layer.label()).color(TEXT_MUTED));
            })
            .response
            .on_hover_text(reason);
        }
    });
}

fn layer_row(
    ui: &mut egui::Ui,
    state: &mut EditorUiState,
    capabilities: &LayerCapabilities,
    layer: WorldLayer,
) {
    let availability = capabilities.availability(layer);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
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
