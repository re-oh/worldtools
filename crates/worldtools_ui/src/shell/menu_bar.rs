use bevy::prelude::MessageWriter;
use bevy_egui::egui::{self, Align, Layout};
use egui_phosphor_icons::{Icon, icons};

use crate::{
    DebugUiState, DocumentStatus, EditorCommand, EditorUiState, GenerationActivity,
    GenerationStatus, SaveState,
    style::{self, BG_HEADER, TEXT_MUTED, TOP_BAR_HEIGHT},
};

pub fn show(
    root: &mut egui::Ui,
    document: &DocumentStatus,
    generation: &GenerationStatus,
    ui_state: &EditorUiState,
    debug_state: &mut DebugUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
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

                file_menu(ui, commands);
                edit_menu(ui, document, commands);
                world_menu(ui, ui_state, generation, commands);
                view_menu(ui, debug_state, commands);

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let busy = matches!(generation.activity, GenerationActivity::Running { .. });
                    if ui
                        .add_enabled(
                            !busy,
                            egui::Button::new(format!("{} GENERATE", icons::PLAY.as_str())),
                        )
                        .on_hover_text("Generate the selected scope")
                        .clicked()
                    {
                        commands.write(EditorCommand::Generate(ui_state.generation_scope));
                    }

                    let marker = match document.save_state {
                        SaveState::Modified => " *",
                        SaveState::Saving => " [saving]",
                        SaveState::Failed => " [save failed]",
                        SaveState::Saved => "",
                    };
                    ui.label(
                        egui::RichText::new(format!("{}{marker}", document.name))
                            .color(TEXT_MUTED)
                            .size(11.0),
                    );
                });
            });
        });
}

fn file_menu(ui: &mut egui::Ui, commands: &mut MessageWriter<EditorCommand>) {
    ui.menu_button("File", |ui| {
        menu_command(
            ui,
            icons::FILE_PLUS,
            "New world",
            EditorCommand::NewWorld,
            commands,
        );
        menu_command(
            ui,
            icons::FOLDER_OPEN,
            "Open...",
            EditorCommand::OpenWorld,
            commands,
        );
        ui.separator();
        menu_command(
            ui,
            icons::FLOPPY_DISK,
            "Save",
            EditorCommand::SaveWorld,
            commands,
        );
        menu_command(
            ui,
            icons::FLOPPY_DISK_BACK,
            "Save as...",
            EditorCommand::SaveWorldAs,
            commands,
        );
    });
}

fn edit_menu(
    ui: &mut egui::Ui,
    document: &DocumentStatus,
    commands: &mut MessageWriter<EditorCommand>,
) {
    ui.menu_button("Edit", |ui| {
        if ui
            .add_enabled(
                document.can_undo,
                egui::Button::new(format!("{} Undo", icons::ARROW_BEND_UP_LEFT.as_str())),
            )
            .clicked()
        {
            commands.write(EditorCommand::Undo);
            ui.close();
        }
        if ui
            .add_enabled(
                document.can_redo,
                egui::Button::new(format!("{} Redo", icons::ARROW_BEND_UP_RIGHT.as_str())),
            )
            .clicked()
        {
            commands.write(EditorCommand::Redo);
            ui.close();
        }
        ui.separator();
        menu_command(
            ui,
            icons::GEAR,
            "Preferences...",
            EditorCommand::OpenPreferences,
            commands,
        );
    });
}

fn world_menu(
    ui: &mut egui::Ui,
    ui_state: &EditorUiState,
    generation: &GenerationStatus,
    commands: &mut MessageWriter<EditorCommand>,
) {
    ui.menu_button("World", |ui| {
        for scope in crate::GenerationScope::ALL {
            if ui
                .button(format!(
                    "{} Generate {}",
                    icons::PLAY.as_str(),
                    scope.label()
                ))
                .clicked()
            {
                commands.write(EditorCommand::Generate(scope));
                ui.close();
            }
        }
        if !generation.dirty.is_empty()
            && ui
                .button(format!("{} Focus dirty region", icons::CROSSHAIR.as_str()))
                .clicked()
        {
            commands.write(EditorCommand::FocusDirtyRegion);
            ui.close();
        }
        ui.separator();
        ui.label(
            egui::RichText::new(format!("Scope: {}", ui_state.generation_scope.label()))
                .color(TEXT_MUTED),
        );
    });
}

fn view_menu(
    ui: &mut egui::Ui,
    debug_state: &mut DebugUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
    ui.menu_button("View", |ui| {
        if ui
            .checkbox(&mut debug_state.visible, "Diagnostics  F12")
            .clicked()
        {
            ui.close();
        }
        ui.separator();
        menu_command(
            ui,
            icons::SLIDERS_HORIZONTAL,
            "Preferences...",
            EditorCommand::OpenPreferences,
            commands,
        );
    });
}

fn menu_command(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    command: EditorCommand,
    commands: &mut MessageWriter<EditorCommand>,
) {
    if ui.button(format!("{} {label}", icon.as_str())).clicked() {
        commands.write(command);
        ui.close();
    }
}
