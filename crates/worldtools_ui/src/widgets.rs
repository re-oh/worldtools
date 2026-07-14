use bevy_egui::egui::{self, Color32, Response, Sense, Stroke, Ui, Vec2};
use egui_phosphor_icons::Icon;

use crate::style::{BORDER, TEXT_MUTED, icon_text};

pub fn section_header(ui: &mut Ui, label: &str) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label.to_uppercase())
                .size(10.0)
                .color(TEXT_MUTED)
                .strong(),
        );
        let remaining = ui.available_width();
        if remaining > 0.0 {
            let (_, rect) = ui.allocate_space(Vec2::new(remaining, 1.0));
            ui.painter()
                .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, BORDER));
        }
    });
    ui.add_space(2.0);
}

pub fn property_row(ui: &mut Ui, label: &str, add_value: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(22.0);
        ui.add_sized(
            [92.0, 18.0],
            egui::Label::new(egui::RichText::new(label).color(TEXT_MUTED)),
        );
        add_value(ui);
    });
}

pub fn icon_button(ui: &mut Ui, icon: Icon, tooltip: &str, size: Vec2, selected: bool) -> Response {
    let text_color = if selected {
        crate::style::ACCENT
    } else {
        crate::style::TEXT
    };
    ui.add_sized(
        size,
        egui::Button::new(icon_text(icon, 17.0, text_color)).selected(selected),
    )
    .on_hover_text(tooltip)
}

pub fn status_indicator(ui: &mut Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, color);
}
