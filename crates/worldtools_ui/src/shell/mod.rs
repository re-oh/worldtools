mod bottom_drawer;
mod debug_window;
mod explorer;
mod inspector;
mod legend;
mod menu_bar;
mod status_bar;
mod tool_rail;
mod viewport;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    AnalysisStatus, DebugCommand, DebugEventLog, DebugTelemetry, DebugUiState, DocumentStatus,
    EditorUiState, GenerationStatus, LayerCapabilities, MapPresentationSettings, MapProbe,
    MapReadout, MapViewport, RegenerateWorld, WorldGenerationDraft, style,
};

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)] // Bevy system parameters are value wrappers.
pub fn draw_editor_shell(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<EditorUiState>,
    document_and_probe: (Res<DocumentStatus>, Res<MapProbe>),
    generation: Res<GenerationStatus>,
    mut generation_draft: ResMut<WorldGenerationDraft>,
    mut presentation: ResMut<MapPresentationSettings>,
    readout: Res<MapReadout>,
    analysis: Res<AnalysisStatus>,
    capabilities: Res<LayerCapabilities>,
    telemetry: Res<DebugTelemetry>,
    events: Res<DebugEventLog>,
    mut debug_state: ResMut<DebugUiState>,
    mut viewport_state: ResMut<MapViewport>,
    mut regeneration: MessageWriter<RegenerateWorld>,
    mut debug_commands: MessageWriter<DebugCommand>,
    mut initialized: Local<bool>,
) -> Result {
    let (document, probe) = document_and_probe;
    let ctx = contexts.ctx_mut()?;
    if !*initialized {
        style::install(ctx);
        *initialized = true;
    }

    handle_shortcuts(ctx, &mut ui_state, &mut debug_state);

    let mut root_ui = egui::Ui::new(
        ctx.clone(),
        "worldtools_editor_root".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    menu_bar::show(&mut root_ui, &document, &mut debug_state);
    status_bar::show(&mut root_ui, &generation, &readout);
    bottom_drawer::show(&mut root_ui, &mut ui_state, &analysis);
    tool_rail::show(&mut root_ui, &mut ui_state);
    explorer::show(
        &mut root_ui,
        &mut ui_state,
        &document,
        &mut generation_draft,
        &generation,
        &capabilities,
        &mut regeneration,
    );
    inspector::show(
        &mut root_ui,
        &mut ui_state,
        &capabilities,
        &probe,
        &mut presentation,
    );
    viewport::show(
        &mut root_ui,
        &mut ui_state,
        &mut viewport_state,
        &presentation,
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
    ui_state: &mut EditorUiState,
    debug_state: &mut DebugUiState,
) {
    if ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::F12)) {
        debug_state.visible = !debug_state.visible;
    }

    if ctx.egui_wants_keyboard_input() {
        return;
    }

    ctx.input_mut(|input| {
        let tool = if input.key_pressed(egui::Key::H) {
            Some(crate::ActiveTool::Navigate)
        } else if input.key_pressed(egui::Key::I) {
            Some(crate::ActiveTool::Inspect)
        } else {
            None
        };
        if let Some(tool) = tool {
            ui_state.active_tool = tool;
        }
    });
}
