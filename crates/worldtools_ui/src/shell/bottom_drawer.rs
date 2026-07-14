use bevy_egui::egui::{self, Align, Layout};
use egui_phosphor_icons::icons;

use crate::{
    AnalysisSeverity, AnalysisStatus, EditorUiState,
    style::{
        self, BG_PANEL, DRAWER_HEADER_HEIGHT, DRAWER_OPEN_HEIGHT, ERROR, TEXT_MUTED, VALID, WARNING,
    },
    widgets,
};

pub fn show(root: &mut egui::Ui, state: &mut EditorUiState, analysis: &AnalysisStatus) {
    let height = if state.analysis_open {
        DRAWER_OPEN_HEIGHT
    } else {
        DRAWER_HEADER_HEIGHT
    };
    egui::Panel::bottom("worldtools_analysis_drawer")
        .exact_size(height)
        .resizable(false)
        .frame(style::panel_frame(BG_PANEL).inner_margin(egui::Margin::symmetric(5, 2)))
        .show(root, |ui| {
            drawer_header(ui, state, analysis);
            if state.analysis_open {
                ui.separator();
                analysis_view(ui, analysis);
            }
        });
}

fn drawer_header(ui: &mut egui::Ui, state: &mut EditorUiState, analysis: &AnalysisStatus) {
    ui.horizontal(|ui| {
        let issue_count = analysis.issues.len();
        let label = if issue_count == 0 {
            format!("{} ANALYSIS", icons::CHART_LINE.as_str())
        } else {
            format!("{} ANALYSIS ({issue_count})", icons::CHART_LINE.as_str())
        };
        if ui.selectable_label(state.analysis_open, label).clicked() {
            state.analysis_open = !state.analysis_open;
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let caret = if state.analysis_open {
                icons::CARET_DOWN
            } else {
                icons::CARET_UP
            };
            if ui
                .button(style::icon_text(caret, 13.0, TEXT_MUTED))
                .on_hover_text(if state.analysis_open {
                    "Collapse analysis"
                } else {
                    "Expand analysis"
                })
                .clicked()
            {
                state.analysis_open = !state.analysis_open;
            }
        });
    });
}

fn analysis_view(ui: &mut egui::Ui, analysis: &AnalysisStatus) {
    if let Some(name) = &analysis.report_name {
        ui.label(egui::RichText::new(name).strong());
    }
    if analysis.issues.is_empty() {
        ui.horizontal(|ui| {
            widgets::status_indicator(ui, VALID);
            ui.label(egui::RichText::new("No reported quality issues").color(TEXT_MUTED));
        });
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for issue in &analysis.issues {
            let (icon, color) = match issue.severity {
                AnalysisSeverity::Information => (icons::INFO, style::ACCENT),
                AnalysisSeverity::Warning => (icons::WARNING, WARNING),
                AnalysisSeverity::Error => (icons::WARNING_OCTAGON, ERROR),
            };
            ui.horizontal(|ui| {
                ui.label(style::icon_text(icon, 13.0, color));
                ui.label(&issue.label);
                if let Some(location) = &issue.location {
                    ui.label(egui::RichText::new(location).color(TEXT_MUTED));
                }
            });
        }
    });
}
