use bevy_egui::egui;
use egui_phosphor_icons::icons;

use crate::{
    ActiveTool, EditorUiState, LayerCapabilities, MapPresentationSettings, MapProbe,
    style::{self, BG_PANEL, INSPECTOR_WIDTH, TEXT_MUTED},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    capabilities: &LayerCapabilities,
    probe: &MapProbe,
    presentation: &mut MapPresentationSettings,
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

            presentation_controls(ui, state, presentation);

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

fn presentation_controls(
    ui: &mut egui::Ui,
    state: &EditorUiState,
    presentation: &mut MapPresentationSettings,
) {
    widgets::section_header(ui, "Presentation");
    ui.horizontal(|ui| {
        ui.label(style::icon_text(
            icons::SLIDERS_HORIZONTAL,
            14.0,
            style::ACCENT,
        ));
        ui.label(egui::RichText::new("Layer appearance").strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::icon_button(
                ui,
                icons::ARROW_COUNTER_CLOCKWISE,
                "Reset this layer's appearance",
                egui::Vec2::splat(22.0),
                false,
            )
            .clicked()
            {
                presentation.reset_layer(state.active_layer);
            }
        });
    });
    let visual = presentation.style_mut(state.active_layer);
    presentation_slider(ui, "Opacity", &mut visual.opacity, 0.0..=1.0);
    presentation_slider(ui, "Relief", &mut visual.relief, 0.0..=1.5);
    presentation_slider(ui, "Shadows", &mut visual.shadows, 0.0..=1.0);
    presentation_slider(ui, "Detail", &mut visual.detail, 0.0..=1.5);
    if matches!(
        state.active_layer,
        crate::WorldLayer::Tectonics
            | crate::WorldLayer::Soil
            | crate::WorldLayer::Vegetation
            | crate::WorldLayer::Geology
    ) {
        presentation_slider(ui, "Borders", &mut visual.boundaries, 0.0..=1.0);
    }
    widgets::property_row(ui, "Labels", |ui| {
        ui.checkbox(&mut visual.show_labels, "Visible");
    });

    widgets::section_header(ui, "Lighting");
    presentation_slider(
        ui,
        "Sun angle",
        &mut presentation.sun_azimuth_degrees,
        0.0..=360.0,
    );
    presentation_slider(
        ui,
        "Sun height",
        &mut presentation.sun_elevation_degrees,
        8.0..=75.0,
    );
    presentation_slider(
        ui,
        "Ambient",
        &mut presentation.ambient_occlusion,
        0.0..=1.0,
    );
    widgets::property_row(ui, "Legend", |ui| {
        ui.checkbox(&mut presentation.legend_visible, "Visible");
    });
}

fn presentation_slider(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
) {
    widgets::property_row(ui, label, |ui| {
        ui.add(
            egui::Slider::new(value, range)
                .show_value(true)
                .max_decimals(2),
        );
    });
}
