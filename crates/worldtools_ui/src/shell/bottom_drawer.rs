use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Align, Layout};
use egui_phosphor_icons::icons;

use crate::{
    AnalysisSeverity, AnalysisStatus, DrawerTab, EditorCommand, EditorUiState, JobQueue, JobState,
    style::{
        self, BG_PANEL, DRAWER_HEADER_HEIGHT, DRAWER_OPEN_HEIGHT, ERROR, TEXT_MUTED, VALID, WARNING,
    },
    widgets,
};

pub fn show(
    root: &mut egui::Ui,
    state: &mut EditorUiState,
    jobs: &JobQueue,
    analysis: &AnalysisStatus,
    commands: &mut MessageWriter<EditorCommand>,
) {
    let height = if state.drawer_open {
        DRAWER_OPEN_HEIGHT
    } else {
        DRAWER_HEADER_HEIGHT
    };
    egui::Panel::bottom("worldtools_bottom_drawer")
        .exact_size(height)
        .resizable(false)
        .frame(style::panel_frame(BG_PANEL).inner_margin(egui::Margin::symmetric(5, 2)))
        .show(root, |ui| {
            drawer_tabs(ui, state, jobs, analysis);
            if state.drawer_open {
                ui.separator();
                match state.drawer_tab {
                    DrawerTab::Jobs => jobs_view(ui, jobs, commands),
                    DrawerTab::Analysis => analysis_view(ui, analysis),
                }
            }
        });
}

fn drawer_tabs(
    ui: &mut egui::Ui,
    state: &mut EditorUiState,
    jobs: &JobQueue,
    analysis: &AnalysisStatus,
) {
    ui.horizontal(|ui| {
        let active_jobs = jobs.active_count();
        let jobs_label = if active_jobs == 0 {
            format!("{} JOBS", icons::QUEUE.as_str())
        } else {
            format!("{} JOBS ({active_jobs})", icons::QUEUE.as_str())
        };
        if ui
            .selectable_label(state.drawer_tab == DrawerTab::Jobs, jobs_label)
            .clicked()
        {
            if state.drawer_tab == DrawerTab::Jobs {
                state.drawer_open = !state.drawer_open;
            } else {
                state.drawer_tab = DrawerTab::Jobs;
                state.drawer_open = true;
            }
        }

        let issue_count = analysis.issues.len();
        let analysis_label = if issue_count == 0 {
            format!("{} ANALYSIS", icons::CHART_LINE.as_str())
        } else {
            format!("{} ANALYSIS ({issue_count})", icons::CHART_LINE.as_str())
        };
        if ui
            .selectable_label(state.drawer_tab == DrawerTab::Analysis, analysis_label)
            .clicked()
        {
            if state.drawer_tab == DrawerTab::Analysis {
                state.drawer_open = !state.drawer_open;
            } else {
                state.drawer_tab = DrawerTab::Analysis;
                state.drawer_open = true;
            }
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let caret = if state.drawer_open {
                icons::CARET_DOWN
            } else {
                icons::CARET_UP
            };
            if ui
                .button(style::icon_text(caret, 13.0, TEXT_MUTED))
                .on_hover_text(if state.drawer_open {
                    "Collapse drawer"
                } else {
                    "Expand drawer"
                })
                .clicked()
            {
                state.drawer_open = !state.drawer_open;
            }
        });
    });
}

fn jobs_view(ui: &mut egui::Ui, jobs: &JobQueue, commands: &mut MessageWriter<EditorCommand>) {
    if jobs.jobs.is_empty() {
        ui.label(egui::RichText::new("No queued work").color(TEXT_MUTED));
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for job in &jobs.jobs {
            ui.horizontal(|ui| {
                let (icon, color, detail) = match &job.state {
                    JobState::Queued => (icons::CIRCLE, TEXT_MUTED, "queued".to_owned()),
                    JobState::Running { progress } => (
                        icons::SPINNER_GAP,
                        style::ACCENT,
                        format!("{:>3.0}%", progress.clamp(0.0, 1.0) * 100.0),
                    ),
                    JobState::Complete => (icons::CHECK, VALID, "complete".to_owned()),
                    JobState::Failed { message } => (icons::WARNING, ERROR, message.clone()),
                };
                ui.label(style::icon_text(icon, 13.0, color));
                ui.add_sized([220.0, 18.0], egui::Label::new(&job.label));
                ui.label(egui::RichText::new(detail).color(color));
                if matches!(job.state, JobState::Queued | JobState::Running { .. })
                    && ui
                        .small_button(icons::X.as_str())
                        .on_hover_text("Cancel job")
                        .clicked()
                {
                    commands.write(EditorCommand::CancelJob(job.id));
                }
            });
        }
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
