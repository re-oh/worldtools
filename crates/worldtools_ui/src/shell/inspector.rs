use bevy_egui::egui;

use crate::{
    ActiveTool, EditorUiState, LayerCapabilities, MapProbe,
    style::{self, BG_PANEL, INSPECTOR_WIDTH, TEXT_MUTED},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    capabilities: &LayerCapabilities,
    probe: &MapProbe,
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
                let available = crate::WorldLayer::ALL
                    .into_iter()
                    .filter(|layer| capabilities.availability(*layer).is_available())
                    .collect::<Vec<_>>();
                if available.len() <= 1 {
                    ui.label(state.active_layer.label());
                } else {
                    egui::ComboBox::from_id_salt("inspector_active_layer")
                        .selected_text(state.active_layer.label())
                        .show_ui(ui, |ui| {
                            for layer in available {
                                ui.selectable_value(&mut state.active_layer, layer, layer.label());
                            }
                        });
                }
            });

            if state.active_tool == ActiveTool::Inspect {
                widgets::section_header(ui, "Sample");
                if let Some(sample) = probe.selected_for(state.active_layer) {
                    widgets::property_row(ui, "Latitude", |ui| {
                        ui.monospace(format!("{:+.4} deg", sample.latitude_degrees));
                    });
                    widgets::property_row(ui, "Longitude", |ui| {
                        ui.monospace(format!("{:+.4} deg", sample.longitude_degrees));
                    });
                    for reading in &sample.readings {
                        widgets::property_row(ui, &reading.label, |ui| {
                            ui.monospace(&reading.value);
                        });
                    }
                } else {
                    ui.label(
                        egui::RichText::new(format!(
                            "Click the map to pin a {} sample",
                            state.active_layer.label().to_lowercase()
                        ))
                        .color(TEXT_MUTED)
                        .italics(),
                    );
                }
            } else {
                ui.label(egui::RichText::new(state.active_layer.description()).color(TEXT_MUTED));
            }
        });
}
