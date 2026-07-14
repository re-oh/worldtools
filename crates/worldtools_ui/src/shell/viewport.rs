use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Order, Sense, Vec2};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    EditorCommand, EditorUiState, MapViewMode, MapViewport, MapViewportChanged, ViewportRect, style,
};

pub fn show(
    root: &mut egui::Ui,
    editor: &mut EditorUiState,
    viewport: &mut MapViewport,
    commands: &mut MessageWriter<EditorCommand>,
    changes: &mut MessageWriter<MapViewportChanged>,
) {
    let ctx = root.ctx().clone();
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(root, |ui| {
            let rect = ui.max_rect();
            let response = ui.allocate_rect(rect, Sense::hover());
            view_selector(&ctx, rect.min + Vec2::splat(8.0), editor, commands);
            let pixels_per_point = ctx.pixels_per_point();
            let logical = ViewportRect {
                min: [rect.min.x, rect.min.y],
                max: [rect.max.x, rect.max.y],
            };
            let physical = logical.physical(pixels_per_point);
            let changed = viewport.logical != logical
                || viewport.physical != physical
                || (viewport.pixels_per_point - pixels_per_point).abs() > f32::EPSILON;

            viewport.logical = logical;
            viewport.physical = physical;
            viewport.pixels_per_point = pixels_per_point;
            viewport.hovered = response.hovered();
            viewport.input_blocked = !response.hovered() || ctx.egui_is_using_pointer();
            viewport.frame = viewport.frame.wrapping_add(1);

            if changed {
                changes.write(MapViewportChanged {
                    logical,
                    physical,
                    pixels_per_point,
                });
            }
        });
}

fn view_selector(
    ctx: &egui::Context,
    position: egui::Pos2,
    editor: &mut EditorUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
    egui::Area::new("worldtools_map_view_selector".into())
        .order(Order::Foreground)
        .fixed_pos(position)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(style::BG_PANEL)
                .stroke(egui::Stroke::new(1.0, style::BORDER))
                .inner_margin(egui::Margin::same(2))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(1.0, 0.0);
                        for mode in MapViewMode::ALL {
                            let selected = editor.map_view == mode;
                            let response = ui
                                .add_sized(
                                    [88.0, 24.0],
                                    egui::Button::new(format!(
                                        "{} {}",
                                        view_icon(mode).as_str(),
                                        mode.label()
                                    ))
                                    .selected(selected),
                                )
                                .on_hover_text(mode.description());
                            if response.clicked() && !selected {
                                editor.map_view = mode;
                                commands.write(EditorCommand::SelectMapView(mode));
                            }
                        }
                    });
                });
        });
}

fn view_icon(mode: MapViewMode) -> Icon {
    match mode {
        MapViewMode::Terrain => icons::MOUNTAINS,
        MapViewMode::Elevation => icons::CHART_LINE,
        MapViewMode::Slope => icons::CARET_UP,
    }
}
