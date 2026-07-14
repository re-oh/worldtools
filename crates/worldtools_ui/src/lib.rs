//! Native editor shell for `WorldTools`.
//!
//! This crate owns editor interaction and layout, but deliberately knows nothing
//! about terrain storage or rendering. Other plugins communicate through the
//! resources and messages re-exported by [`prelude`].

mod model;
mod shell;
mod style;
mod widgets;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub use model::*;

/// Installs the `WorldTools` editor shell and its public communication contract.
pub struct WorldToolsUiPlugin;

impl Plugin for WorldToolsUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.init_resource::<EditorUiState>()
            .init_resource::<DebugUiState>()
            .init_resource::<DebugTelemetry>()
            .init_resource::<DebugEventLog>()
            .init_resource::<LayerCapabilities>()
            .init_resource::<DocumentStatus>()
            .init_resource::<GenerationStatus>()
            .init_resource::<MapReadout>()
            .init_resource::<MapProbe>()
            .init_resource::<JobQueue>()
            .init_resource::<AnalysisStatus>()
            .init_resource::<MapViewport>()
            .add_message::<EditorCommand>()
            .add_message::<DebugCommand>()
            .add_message::<MapViewportChanged>()
            .add_systems(EguiPrimaryContextPass, shell::draw_editor_shell);
    }
}

/// Common imports for plugins integrating with the editor shell.
pub mod prelude {
    pub use crate::{
        ActiveTool, AnalysisIssue, AnalysisSeverity, AnalysisStatus, BrushFalloff, BrushOperation,
        BrushSettings, DebugCommand, DebugEvent, DebugEventLevel, DebugEventLog,
        DebugRenderOptions, DebugTab, DebugTelemetry, DebugUiState, DirtyRegion, DocumentStatus,
        DrawerTab, EditorCommand, EditorUiState, FrameDiagnostics, GenerationActivity,
        GenerationScope, GenerationStatus, JobId, JobQueue, JobState, JobSummary,
        LayerAvailability, LayerCapabilities, LayerProbe, MapProbe, MapReadout, MapViewMode,
        MapViewport, MapViewportChanged, PipelineStage, ProbeReading, RenderDiagnostics, SaveState,
        StreamingDiagnostics, TerrainProbe, ViewportDiagnostics, ViewportRect, WorldLayer,
        WorldToolsUiPlugin,
    };
}
