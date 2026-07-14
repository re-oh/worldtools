use bevy_egui::egui::{self, Align, Layout};

use crate::{
    DocumentStatus, GenerationActivity, GenerationStatus, MapReadout, SaveState,
    style::{self, BG_HEADER, ERROR, STATUS_BAR_HEIGHT, TEXT_MUTED, VALID, WARNING},
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    document: &DocumentStatus,
    generation: &GenerationStatus,
    readout: &MapReadout,
) {
    egui::Panel::bottom("worldtools_status_bar")
        .exact_size(STATUS_BAR_HEIGHT)
        .frame(style::panel_frame(BG_HEADER).inner_margin(egui::Margin::symmetric(6, 2)))
        .show(root, |ui| {
            ui.horizontal_centered(|ui| {
                let (color, text) = generation_label(generation);
                widgets::status_indicator(ui, color);
                ui.label(egui::RichText::new(text).color(TEXT_MUTED).size(11.0));

                if !generation.dirty.is_empty() {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "{} dirty tile{}",
                            generation.dirty.tile_count,
                            if generation.dirty.tile_count == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ))
                        .color(WARNING)
                        .size(11.0),
                    );
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let save_label = match document.save_state {
                        SaveState::Saved => "SAVED",
                        SaveState::Modified => "MODIFIED",
                        SaveState::Saving => "SAVING",
                        SaveState::Failed => "SAVE FAILED",
                    };
                    ui.label(egui::RichText::new(save_label).color(TEXT_MUTED).size(10.0));

                    if readout.meters_per_pixel > 0.0 {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "LOD {} | {}",
                                readout.lod,
                                format_resolution(readout.meters_per_pixel)
                            ))
                            .color(TEXT_MUTED)
                            .size(11.0),
                        );
                    }

                    if let Some([longitude, latitude]) = readout.cursor_degrees {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!(
                                "{latitude:+08.3} deg  {longitude:+09.3} deg"
                            ))
                            .color(TEXT_MUTED)
                            .size(11.0),
                        );
                    }
                });
            });
        });
}

fn generation_label(status: &GenerationStatus) -> (egui::Color32, String) {
    match &status.activity {
        GenerationActivity::Idle => (VALID, "Ready".to_owned()),
        GenerationActivity::Queued { jobs } => (WARNING, format!("Queued | {jobs} jobs")),
        GenerationActivity::Running {
            stage,
            completed,
            total,
        } => (
            style::ACCENT,
            format!("Generating {stage} | {completed}/{total}"),
        ),
        GenerationActivity::Failed { message } => (ERROR, format!("Generation failed | {message}")),
    }
}

fn format_resolution(meters_per_pixel: f64) -> String {
    if meters_per_pixel >= 1_000.0 {
        format!("{:.1} km/px", meters_per_pixel / 1_000.0)
    } else if meters_per_pixel >= 10.0 {
        format!("{meters_per_pixel:.0} m/px")
    } else {
        format!("{meters_per_pixel:.1} m/px")
    }
}

#[cfg(test)]
mod tests {
    use super::format_resolution;

    #[test]
    fn resolution_uses_a_readable_unit() {
        assert_eq!(format_resolution(1.25), "1.2 m/px");
        assert_eq!(format_resolution(32.0), "32 m/px");
        assert_eq!(format_resolution(4_200.0), "4.2 km/px");
    }
}
