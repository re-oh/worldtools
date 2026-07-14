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
    match layer {
        WorldLayer::Elevation => {}
        WorldLayer::Tectonics => {
            probe.push_reading("Plate", format!("#{}", sample.tectonics.plate_id));
            probe.push_reading("Crust", sample.tectonics.crust.label());
            probe.push_reading(
                "Crust age",
                format!("{:.0} Myr", sample.tectonics.crust_age_myr),
            );
            probe.push_reading("Boundary", percent(sample.tectonics.boundary));
            probe.push_reading("Convergence", percent(sample.tectonics.convergence));
            probe.push_reading("Divergence", percent(sample.tectonics.divergence));
            probe.push_reading("Uplift", format!("{:+.0} m", sample.tectonics.uplift_m));
            probe.push_reading("Volcanism", percent(sample.tectonics.volcanism));
        }
        WorldLayer::Hydrology => {
            probe.push_reading("River", percent(sample.hydrology.river_strength));
            probe.push_reading("Runoff", percent(sample.hydrology.runoff));
            probe.push_reading("Wetness", percent(sample.hydrology.wetness));
            probe.push_reading("Lake", percent(sample.hydrology.lake));
            probe.push_reading("Erosion", format!("{:.1} m", sample.hydrology.erosion_m));
            probe.push_reading("Sediment", format!("{:.1} m", sample.hydrology.sediment_m));
        }
        WorldLayer::Climate => {
            probe.push_reading("Climate", sample.climate.zone.label());
            probe.push_reading(
                "Temperature",
                format!("{:+.1} C", sample.climate.temperature_c),
            );
            probe.push_reading(
                "Precipitation",
                format!("{:.0} mm/yr", sample.climate.precipitation_mm),
            );
            probe.push_reading("Seasonality", percent(sample.climate.seasonality));
            probe.push_reading(
                "Wind",
                wind_reading(sample.climate.wind_east, sample.climate.wind_north),
            );
            probe.push_reading("Aridity", percent(sample.climate.aridity));
        }
        WorldLayer::Soil => {
            probe.push_reading("Soil", sample.soil.kind.label());
            probe.push_reading("Depth", format!("{:.2} m", sample.soil.depth_m));
            probe.push_reading("Fertility", percent(sample.soil.fertility));
            probe.push_reading("Clay", percent(sample.soil.clay_fraction));
            probe.push_reading("Organic", percent(sample.soil.organic_fraction));
            probe.push_reading("Drainage", percent(sample.soil.drainage));
        }
        WorldLayer::Vegetation => {
            probe.push_reading("Biome", sample.vegetation.biome.label());
            probe.push_reading("Canopy", percent(sample.vegetation.canopy_fraction));
            probe.push_reading("Grass", percent(sample.vegetation.grass_fraction));
            probe.push_reading("Biomass", percent(sample.vegetation.biomass));
            probe.push_reading("Fire frequency", percent(sample.vegetation.fire_frequency));
        }
        WorldLayer::Geology => {
            probe.push_reading("Bedrock", sample.geology.lithology.label());
            probe.push_reading(
                "Rock age",
                format!("{:.0} Myr", sample.geology.rock_age_myr),
            );
            probe.push_reading("Sediment", format!("{:.1} m", sample.geology.sediment_m));
            probe.push_reading(
                "Volcanic ash",
                format!("{:.2} m", sample.geology.volcanic_ash_m),
            );
            probe.push_reading("Weathering", percent(sample.geology.weathering));
        }
        WorldLayer::Resources => {
            probe.push_reading("Deposit", sample.resources.dominant.label());
            probe.push_reading("Richness", percent(sample.resources.richness));
            probe.push_reading("Depth", format!("{:.0} m", sample.resources.depth_m));
            probe.push_reading("Confidence", percent(sample.resources.confidence));
            probe.push_reading("Metallic", percent(sample.resources.metallic));
            probe.push_reading("Energy", percent(sample.resources.energy));
            probe.push_reading("Industrial", percent(sample.resources.industrial));
        }
    }
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
