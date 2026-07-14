mod audit;
mod commands;
mod io;
mod snapshot;
mod telemetry;

use bevy::{
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};

pub struct WorldToolsDebugPlugin;

impl Plugin for WorldToolsDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin,
        ))
        .init_resource::<audit::AuditRuntime>()
        .add_message::<audit::AuditRequest>()
        .add_message::<snapshot::SnapshotRequest>()
        .add_systems(
            Update,
            (
                telemetry::drain_trace_events,
                telemetry::sync_telemetry,
                commands::handle_debug_commands,
                snapshot::capture_snapshots,
                audit::start_audits,
                audit::receive_audits,
            )
                .chain(),
        );
    }
}
