mod debug_tools;
mod diagnostics;
mod generation;
mod interaction;
mod layers;
#[cfg(feature = "live-debug")]
mod live_remote;
mod viewport_bridge;

use bevy::{prelude::*, window::PresentMode};
use worldtools_render::WorldToolsRenderPlugin;
use worldtools_ui::WorldToolsUiPlugin;

fn main() {
    let diagnostics = diagnostics::Diagnostics::install();
    let mut app = App::new();
    app.insert_resource(diagnostics.event_receiver())
        .insert_resource(diagnostics.directory_resource())
        .insert_resource(ClearColor(Color::srgb(0.055, 0.059, 0.061)))
        .add_plugins(
            DefaultPlugins
                .set(diagnostics.log_plugin())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "WorldTools".into(),
                        resolution: (1600, 900).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            WorldToolsRenderPlugin,
            WorldToolsUiPlugin,
            generation::WorldGenerationPlugin,
            interaction::WorldInteractionPlugin,
            viewport_bridge::ViewportBridgePlugin,
            debug_tools::WorldToolsDebugPlugin,
        ));

    #[cfg(feature = "live-debug")]
    app.add_plugins(live_remote::WorldToolsRemoteDebugPlugin);

    diagnostics.record_startup();
    tracing::debug!(
        target: "worldtools::diagnostics",
        directory = %diagnostics.directory().display(),
        "Diagnostics files are available"
    );
    app.run();
}
