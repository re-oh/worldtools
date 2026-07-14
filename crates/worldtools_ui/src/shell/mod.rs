mod bottom_drawer;
mod debug_window;
mod explorer;
mod inspector;
mod menu_bar;
mod status_bar;
mod tool_rail;
mod viewport;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    AnalysisStatus, DebugCommand, DebugEventLog, DebugTelemetry, DebugUiState, DocumentStatus,
    EditorCommand, EditorUiState, GenerationStatus, JobQueue, LayerCapabilities, MapProbe,
    MapReadout, MapViewport, MapViewportChanged, style,
};

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)] // Bevy system parameters are value wrappers.
pub fn draw_editor_shell(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    document_and_probe: (Res<DocumentStatus>, Res<MapProbe>),
    generation: Res<GenerationStatus>,
    readout: Res<MapReadout>,
    jobs: Res<JobQueue>,
    analysis: Res<AnalysisStatus>,
    capabilities: Res<LayerCapabilities>,
    telemetry: Res<DebugTelemetry>,
    events: Res<DebugEventLog>,
    mut debug_state: ResMut<DebugUiState>,
    mut viewport_state: ResMut<MapViewport>,
    mut commands: MessageWriter<EditorCommand>,
    mut debug_commands: MessageWriter<DebugCommand>,
    mut viewport_changes: MessageWriter<MapViewportChanged>,
    mut initialized: Local<bool>,
) -> Result {
    let (document, probe) = document_and_probe;
    let ctx = contexts.ctx_mut()?;
    if !*initialized {
        style::install(ctx);
        *initialized = true;
    }

    handle_shortcuts(
        ctx,
        &document,
        &mut ui_state,
        &mut debug_state,
        &mut commands,
    );

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "worldtools_editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    menu_bar::show(
        &mut root_ui,
        &document,
        &generation,
        &ui_state,
        &mut debug_state,
        &mut commands,
    );
    status_bar::show(&mut root_ui, &document, &generation, &readout);
    bottom_drawer::show(&mut root_ui, &mut ui_state, &jobs, &analysis, &mut commands);
    tool_rail::show(&mut root_ui, &mut ui_state, &mut commands);
    explorer::show(
        &mut root_ui,
        &mut ui_state,
        &document,
        &capabilities,
        &mut commands,
    );
    inspector::show(
        &mut root_ui,
        &mut ui_state,
        &capabilities,
        &probe,
        &mut commands,
    );
    viewport::show(
        &mut root_ui,
        &mut ui_state,
        &mut viewport_state,
        &mut commands,
        &mut viewport_changes,
    );
    debug_window::show(
        ctx,
        &mut debug_state,
        &telemetry,
        &events,
        &capabilities,
        &ui_state,
        &generation,
        &mut debug_commands,
    );

    Ok(())
}

fn handle_shortcuts(
    ctx: &egui::Context,
    document: &DocumentStatus,
    ui_state: &mut EditorUiState,
    debug_state: &mut DebugUiState,
    commands: &mut MessageWriter<EditorCommand>,
) {
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F12)) {
        debug_state.visible = !debug_state.visible;
    }

    if ctx.egui_wants_keyboard_input() {
        return;
    }

    ctx.input_mut(|input| {
        let command = egui::Modifiers::COMMAND;
        let shift_command = egui::Modifiers {
            shift: true,
            ..command
        };

        if input.consume_shortcut(&egui::KeyboardShortcut::new(command, egui::Key::N)) {
            commands.write(EditorCommand::NewWorld);
        }
        if input.consume_shortcut(&egui::KeyboardShortcut::new(command, egui::Key::O)) {
            commands.write(EditorCommand::OpenWorld);
        }
        if input.consume_shortcut(&egui::KeyboardShortcut::new(shift_command, egui::Key::S)) {
            commands.write(EditorCommand::SaveWorldAs);
        }
        if input.consume_shortcut(&egui::KeyboardShortcut::new(command, egui::Key::S)) {
            commands.write(EditorCommand::SaveWorld);
        }
        if document.can_undo
            && input.consume_shortcut(&egui::KeyboardShortcut::new(command, egui::Key::Z))
        {
            commands.write(EditorCommand::Undo);
        }
        if document.can_redo
            && input.consume_shortcut(&egui::KeyboardShortcut::new(command, egui::Key::Y))
        {
            commands.write(EditorCommand::Redo);
        }

        let tool = if input.key_pressed(egui::Key::H) {
            Some(crate::ActiveTool::Navigate)
        } else if input.key_pressed(egui::Key::I) {
            Some(crate::ActiveTool::Inspect)
        } else if input.key_pressed(egui::Key::B) {
            Some(crate::ActiveTool::Sculpt)
        } else {
            None
        };
        if let Some(tool) = tool {
            ui_state.active_tool = tool;
            commands.write(EditorCommand::SelectTool(tool));
        }
    });
}
