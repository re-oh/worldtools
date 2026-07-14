use bevy::prelude::Resource;

/// A presentation of the currently available elevation dataset.
///
/// These modes never imply additional generated world data. They are derived
/// directly from elevation and its screen-space derivatives in the tile shader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum MapDisplayMode {
    #[default]
    Physical = 0,
    Elevation = 1,
    Relief = 2,
    Slope = 3,
    Contours = 4,
}

impl MapDisplayMode {
    #[must_use]
    pub const fn shader_id(self) -> u32 {
        self as u32
    }

    const fn shader_value(self) -> f32 {
        match self {
            Self::Physical => 0.0,
            Self::Elevation => 1.0,
            Self::Relief => 2.0,
            Self::Slope => 3.0,
            Self::Contours => 4.0,
        }
    }
}

/// Uniform-only controls for presenting map tile data.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct MapDisplaySettings {
    pub mode: MapDisplayMode,
    pub sea_level_m: f32,
    pub contour_interval_m: f32,
    pub relief_strength: f32,
}

impl Default for MapDisplaySettings {
    fn default() -> Self {
        Self {
            mode: MapDisplayMode::Physical,
            sea_level_m: 0.0,
            contour_interval_m: 250.0,
            relief_strength: 1.0,
        }
    }
}

impl MapDisplaySettings {
    #[must_use]
    pub fn sanitized(mut self) -> Self {
        if !self.sea_level_m.is_finite() {
            self.sea_level_m = 0.0;
        }
        if !self.contour_interval_m.is_finite() {
            self.contour_interval_m = 250.0;
        }
        if !self.relief_strength.is_finite() {
            self.relief_strength = 1.0;
        }
        self.contour_interval_m = self.contour_interval_m.clamp(1.0, 10_000.0);
        self.relief_strength = self.relief_strength.clamp(0.0, 2.0);
        self
    }

    #[must_use]
    pub(crate) fn shader_values(self) -> [f32; 4] {
        let settings = self.sanitized();
        [
            settings.mode.shader_value(),
            settings.sea_level_m,
            settings.contour_interval_m,
            settings.relief_strength,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_ids_are_stable() {
        assert_eq!(MapDisplayMode::Physical.shader_id(), 0);
        assert_eq!(MapDisplayMode::Elevation.shader_id(), 1);
        assert_eq!(MapDisplayMode::Relief.shader_id(), 2);
        assert_eq!(MapDisplayMode::Slope.shader_id(), 3);
        assert_eq!(MapDisplayMode::Contours.shader_id(), 4);
    }

    #[test]
    fn invalid_uniform_values_are_sanitized() {
        let settings = MapDisplaySettings {
            sea_level_m: f32::NAN,
            contour_interval_m: f32::INFINITY,
            relief_strength: -5.0,
            ..MapDisplaySettings::default()
        }
        .sanitized();

        assert!(settings.sea_level_m.abs() < f32::EPSILON);
        assert!((settings.contour_interval_m - 250.0).abs() < f32::EPSILON);
        assert!(settings.relief_strength.abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_ranges_are_bounded() {
        let settings = MapDisplaySettings {
            contour_interval_m: 0.01,
            relief_strength: 20.0,
            ..MapDisplaySettings::default()
        }
        .sanitized();

        assert!((settings.contour_interval_m - 1.0).abs() < f32::EPSILON);
        assert!((settings.relief_strength - 2.0).abs() < f32::EPSILON);
    }
}
