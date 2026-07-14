use bevy::prelude::Resource;

use super::WorldLayer;

/// One human-readable value reported by a sampled world-data layer.
///
/// Values are deliberately presentation-ready strings. This keeps the UI
/// independent of the simulation's units, enums, and storage representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProbeReading {
    pub label: String,
    pub value: String,
}

impl ProbeReading {
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// A pinned sample from one world-data layer.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerProbe {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
    pub layer: WorldLayer,
    pub readings: Vec<ProbeReading>,
}

impl LayerProbe {
    #[must_use]
    pub fn new(latitude_degrees: f64, longitude_degrees: f64, layer: WorldLayer) -> Self {
        Self {
            latitude_degrees,
            longitude_degrees,
            layer,
            readings: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_reading(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.readings.push(ProbeReading::new(label, value));
        self
    }

    pub fn push_reading(&mut self, label: impl Into<String>, value: impl Into<String>) {
        self.readings.push(ProbeReading::new(label, value));
    }
}

/// Compatibility input for the elevation sampler used by the native app.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainProbe {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
    pub elevation_m: f32,
    pub slope_degrees: f64,
    pub is_water: bool,
}

impl TerrainProbe {
    #[must_use]
    pub fn into_layer_probe(self) -> LayerProbe {
        self.into()
    }
}

impl From<TerrainProbe> for LayerProbe {
    fn from(sample: TerrainProbe) -> Self {
        Self::new(
            sample.latitude_degrees,
            sample.longitude_degrees,
            WorldLayer::Elevation,
        )
        .with_reading("Elevation", format!("{:+.0} m", sample.elevation_m))
        .with_reading("Slope", format!("{:.1} deg", sample.slope_degrees))
        .with_reading("Surface", if sample.is_water { "Water" } else { "Land" })
    }
}

#[derive(Debug, Default, Resource)]
pub struct MapProbe {
    pub selected: Option<LayerProbe>,
}

impl MapProbe {
    #[must_use]
    pub fn selected_for(&self, layer: WorldLayer) -> Option<&LayerProbe> {
        self.selected
            .as_ref()
            .filter(|sample| sample.layer == layer)
    }

    pub fn select(&mut self, sample: impl Into<LayerProbe>) {
        self.selected = Some(sample.into());
    }

    pub fn select_terrain(&mut self, sample: TerrainProbe) {
        self.select(sample);
    }

    pub fn clear(&mut self) {
        self.selected = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain_sample() -> TerrainProbe {
        TerrainProbe {
            latitude_degrees: 46.25,
            longitude_degrees: -122.18,
            elevation_m: 2_549.4,
            slope_degrees: 17.26,
            is_water: false,
        }
    }

    #[test]
    fn terrain_probe_preserves_the_existing_elevation_readout() {
        let sample = terrain_sample().into_layer_probe();

        assert_eq!(sample.layer, WorldLayer::Elevation);
        assert!((sample.latitude_degrees - 46.25).abs() < f64::EPSILON);
        assert!((sample.longitude_degrees + 122.18).abs() < f64::EPSILON);
        assert_eq!(
            sample.readings,
            [
                ProbeReading::new("Elevation", "+2549 m"),
                ProbeReading::new("Slope", "17.3 deg"),
                ProbeReading::new("Surface", "Land"),
            ]
        );
    }

    #[test]
    fn selections_are_only_visible_to_the_sampled_layer() {
        let mut probe = MapProbe::default();
        probe.select_terrain(terrain_sample());

        assert!(probe.selected_for(WorldLayer::Elevation).is_some());
        assert!(probe.selected_for(WorldLayer::Climate).is_none());
    }

    #[test]
    fn arbitrary_layers_can_report_ordered_domain_readings() {
        let sample = LayerProbe::new(12.0, 34.0, WorldLayer::Climate)
            .with_reading("Mean temperature", "18.2 C")
            .with_reading("Annual precipitation", "1,204 mm")
            .with_reading("Prevailing wind", "ENE 6.4 m/s");

        assert_eq!(sample.readings.len(), 3);
        assert_eq!(sample.readings[1].label, "Annual precipitation");
        assert_eq!(sample.readings[2].value, "ENE 6.4 m/s");
    }

    #[test]
    fn selecting_a_new_layer_replaces_and_clear_removes_the_pin() {
        let mut probe = MapProbe::default();
        probe.select_terrain(terrain_sample());
        probe.select(
            LayerProbe::new(-8.0, 140.0, WorldLayer::Hydrology)
                .with_reading("Drainage area", "42,000 km2"),
        );

        assert!(probe.selected_for(WorldLayer::Elevation).is_none());
        assert!(probe.selected_for(WorldLayer::Hydrology).is_some());

        probe.clear();
        assert!(probe.selected.is_none());
    }
}
