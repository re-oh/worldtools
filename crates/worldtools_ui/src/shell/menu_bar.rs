use bevy_egui::egui::{self, Align, Layout};

use crate::{
    DebugUiState, DocumentStatus,
    style::{self, BG_HEADER, TEXT_MUTED, TOP_BAR_HEIGHT},
};

pub fn show(root: &mut egui::Ui, document: &DocumentStatus, debug_state: &mut DebugUiState) {
    egui::Panel::top("worldtools_menu_bar")
        .exact_size(TOP_BAR_HEIGHT)
        .frame(style::panel_frame(BG_HEADER).inner_margin(egui::Margin::symmetric(7, 4)))
        .show(root, |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(
                    egui::RichText::new("WORLDTOOLS")
                        .size(11.0)
                        .strong()
                        .color(style::ACCENT),
                );
                ui.separator();
                view_menu(ui, debug_state);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{}  |  SEED {}",
                            document.name, document.seed
                        ))
                        .color(TEXT_MUTED)
                        .size(11.0),
                    );
                });
            });
        });
}

fn view_menu(ui: &mut egui::Ui, debug_state: &mut DebugUiState) {
    ui.menu_button("View", |ui| {
        if ui
            .checkbox(&mut debug_state.visible, "Diagnostics  F12")
            .clicked()
        {
            ui.close();
        }
    });
}
