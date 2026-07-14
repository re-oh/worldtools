use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Sense};

use crate::{MapViewport, MapViewportChanged, ViewportRect};

pub fn show(
    root: &mut egui::Ui,
    viewport: &mut MapViewport,
    changes: &mut MessageWriter<MapViewportChanged>,
) {
    let ctx = root.ctx().clone();
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE)
        .show(root, |ui| {
            let rect = ui.max_rect();
            let response = ui.allocate_rect(rect, Sense::hover());
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
