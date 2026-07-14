mod probe;

use bevy::{prelude::*, window::PrimaryWindow};
use worldtools_render::{MapView, TileStreamStats};
use worldtools_ui::{LayerAvailability, LayerCapabilities, MapReadout, MapViewport as UiViewport};
use worldtools_world::GeoPoint;

const PLANET_RADIUS_M: f64 = 6_371_000.0;

pub struct WorldInteractionPlugin;

impl Plugin for WorldInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, publish_layer_capabilities)
            .add_systems(Update, (probe::capture_inspection, sync_readout).chain());
    }
}

fn publish_layer_capabilities(mut capabilities: ResMut<LayerCapabilities>) {
    capabilities.set_all(LayerAvailability::Available);
}

#[allow(clippy::needless_pass_by_value)] // Bevy system parameters are value wrappers.
fn sync_readout(
    stats: Res<TileStreamStats>,
    view: Res<MapView>,
    viewport: Res<UiViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut readout: ResMut<MapReadout>,
) {
    let height = viewport.physical.height().max(1.0);
    readout.lod = stats.level;
    readout.meters_per_pixel =
        std::f64::consts::PI * PLANET_RADIUS_M * f64::from(view.vertical_span) / f64::from(height);
    readout.cursor_degrees = windows
        .single()
        .ok()
        .and_then(|window| pointer_geo(window, &viewport, *view))
        .map(|point| [point.longitude.to_degrees(), point.latitude.to_degrees()]);
}

fn pointer_geo(window: &Window, viewport: &UiViewport, view: MapView) -> Option<GeoPoint> {
    let pointer = window.cursor_position()?;
    let logical = viewport.window_logical(window.scale_factor());
    let min = Vec2::from_array(logical.min);
    let size = Vec2::new(logical.width(), logical.height());
    let local = pointer - min;
    if size.min_element() <= 1.0 || !local.cmpge(Vec2::ZERO).all() || !local.cmple(size).all() {
        return None;
    }
    let aspect = size.x / size.y;
    let normalized = local / size - Vec2::splat(0.5);
    let world_x =
        (view.center.x + f64::from(normalized.x * view.horizontal_span(aspect))).rem_euclid(1.0);
    let world_y = (view.center.y + f64::from(normalized.y * view.vertical_span)).clamp(0.0, 1.0);
    Some(GeoPoint::from_degrees(
        90.0 - 180.0 * world_y,
        -180.0 + 360.0 * world_x,
    ))
}
