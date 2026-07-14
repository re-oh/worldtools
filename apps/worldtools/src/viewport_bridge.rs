use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use worldtools_render::MapViewport as RenderViewport;
use worldtools_ui::MapViewport as UiViewport;

pub struct ViewportBridgePlugin;

impl Plugin for ViewportBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_viewport);
    }
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn sync_viewport(
    ui: Res<UiViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut render: ResMut<RenderViewport>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let logical = ui.window_logical(window.scale_factor());
    let next_min = Vec2::from_array(logical.min);
    let next_max = Vec2::from_array(logical.max);
    if render.min != next_min || render.max != next_max {
        render.min = next_min;
        render.max = next_max;
    }
    render.input_blocked = ui.input_blocked;
    render.pixels_per_point = window.scale_factor();
}
