use bevy::{input::ButtonInput, prelude::*, window::PrimaryWindow};
use worldtools_render::{MapTileStreamer, MapView};
use worldtools_simulation::WorldSample;
use worldtools_ui::{
    ActiveTool, EditorUiState, LayerProbe, MapProbe, MapReadout, MapViewport, TerrainProbe,
    WorldLayer,
};
use worldtools_world::GeoPoint;

use super::{PLANET_RADIUS_M, pointer_geo};

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub(super) fn capture_inspection(
    buttons: Res<ButtonInput<MouseButton>>,
    ui: Res<EditorUiState>,
    viewport: Res<MapViewport>,
    view: Res<MapView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    streamer: Res<MapTileStreamer>,
    readout: Res<MapReadout>,
    mut probe: ResMut<MapProbe>,
) {
    if ui.active_tool != ActiveTool::Inspect || !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(point) = (!viewport.input_blocked)
        .then(|| pointer_geo(window, &viewport, *view))
        .flatten()
    else {
        return;
    };

    let sample = streamer.sample_world(point);
    if ui.active_layer == WorldLayer::Elevation {
        let elevation_m = streamer.sample_elevation(point);
        probe.select_terrain(TerrainProbe {
            latitude_degrees: point.latitude.to_degrees(),
            longitude_degrees: point.longitude.to_degrees(),
            elevation_m,
            slope_degrees: sample_slope(
                point,
                &streamer,
                readout.meters_per_pixel.clamp(1.0, 10_000.0),
            ),
            is_water: elevation_m < 0.0,
        });
    } else {
        probe.select(layer_probe(point, ui.active_layer, sample));
    }
}

fn layer_probe(point: GeoPoint, layer: WorldLayer, sample: WorldSample) -> LayerProbe {
    let mut probe = LayerProbe::new(
        point.latitude.to_degrees(),
        point.longitude.to_degrees(),
        layer,
    );
    format::populate(&mut probe, layer, sample);
    probe
}

fn sample_slope(point: GeoPoint, streamer: &MapTileStreamer, baseline_m: f64) -> f64 {
    let angular = baseline_m / PLANET_RADIUS_M;
    let latitude = point.latitude;
    let longitude = point.longitude;
    let longitude_step = angular / latitude.cos().abs().max(0.05);
    let north = GeoPoint::from_radians(
        (latitude + angular).min(std::f64::consts::FRAC_PI_2),
        longitude,
    );
    let south = GeoPoint::from_radians(
        (latitude - angular).max(-std::f64::consts::FRAC_PI_2),
        longitude,
    );
    let east = GeoPoint::from_radians(latitude, longitude + longitude_step);
    let west = GeoPoint::from_radians(latitude, longitude - longitude_step);
    let dz_north = f64::from(streamer.sample_elevation(north) - streamer.sample_elevation(south));
    let dz_east = f64::from(streamer.sample_elevation(east) - streamer.sample_elevation(west));
    let gradient = dz_north.hypot(dz_east) / (2.0 * baseline_m);
    gradient.atan().to_degrees()
}

fn percent(value: f32) -> String {
    format!("{:.0}%", value.clamp(0.0, 1.0) * 100.0)
}

fn wind_reading(east: f32, north: f32) -> String {
    let speed = east.hypot(north);
    if speed < 0.05 {
        return "Calm".to_owned();
    }
    let angle = east.atan2(north).to_degrees().rem_euclid(360.0);
    let direction = match angle {
        value if !(22.5..337.5).contains(&value) => "N",
        value if value < 67.5 => "NE",
        value if value < 112.5 => "E",
        value if value < 157.5 => "SE",
        value if value < 202.5 => "S",
        value if value < 247.5 => "SW",
        value if value < 292.5 => "W",
        _ => "NW",
    };
    format!("{direction} {speed:.1} m/s")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wind_readout_uses_map_direction_and_speed() {
        assert_eq!(wind_reading(0.0, 0.0), "Calm");
        assert_eq!(wind_reading(5.0, 0.0), "E 5.0 m/s");
        assert_eq!(wind_reading(-3.0, 3.0), "NW 4.2 m/s");
    }

    #[test]
    fn percentages_clamp_untrusted_display_values() {
        assert_eq!(percent(-1.0), "0%");
        assert_eq!(percent(0.456), "46%");
        assert_eq!(percent(2.0), "100%");
    }
}
mod format;
