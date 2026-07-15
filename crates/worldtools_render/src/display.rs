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
    Tectonics = 5,
    Hydrology = 6,
    Climate = 7,
    Soil = 8,
    Vegetation = 9,
    Geology = 10,
    Resources = 11,
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
            Self::Tectonics => 5.0,
            Self::Hydrology => 6.0,
            Self::Climate => 7.0,
            Self::Soil => 8.0,
            Self::Vegetation => 9.0,
            Self::Geology => 10.0,
            Self::Resources => 11.0,
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
    pub shadow_strength: f32,
    pub detail_strength: f32,
    pub boundary_strength: f32,
    pub layer_opacity: f32,
    pub sun_azimuth_degrees: f32,
    pub sun_elevation_degrees: f32,
    pub ambient_occlusion: f32,
}

impl Default for MapDisplaySettings {
    fn default() -> Self {
        Self {
            mode: MapDisplayMode::Physical,
            sea_level_m: 0.0,
            contour_interval_m: 250.0,
            relief_strength: 0.9,
            shadow_strength: 0.62,
            detail_strength: 0.72,
            boundary_strength: 0.72,
            layer_opacity: 0.92,
            sun_azimuth_degrees: 315.0,
            sun_elevation_degrees: 38.0,
            ambient_occlusion: 0.48,
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
        if !self.shadow_strength.is_finite() {
            self.shadow_strength = 0.62;
        }
        if !self.detail_strength.is_finite() {
            self.detail_strength = 0.72;
        }
        if !self.boundary_strength.is_finite() {
            self.boundary_strength = 0.72;
        }
        if !self.layer_opacity.is_finite() {
            self.layer_opacity = 0.92;
        }
        if !self.sun_azimuth_degrees.is_finite() {
            self.sun_azimuth_degrees = 315.0;
        }
        if !self.sun_elevation_degrees.is_finite() {
            self.sun_elevation_degrees = 38.0;
        }
        if !self.ambient_occlusion.is_finite() {
            self.ambient_occlusion = 0.48;
        }
        self.contour_interval_m = self.contour_interval_m.clamp(1.0, 10_000.0);
        self.relief_strength = self.relief_strength.clamp(0.0, 2.0);
        self.shadow_strength = self.shadow_strength.clamp(0.0, 1.0);
        self.detail_strength = self.detail_strength.clamp(0.0, 1.5);
        self.boundary_strength = self.boundary_strength.clamp(0.0, 1.0);
        self.layer_opacity = self.layer_opacity.clamp(0.0, 1.0);
        self.sun_azimuth_degrees = self.sun_azimuth_degrees.rem_euclid(360.0);
        self.sun_elevation_degrees = self.sun_elevation_degrees.clamp(5.0, 85.0);
        self.ambient_occlusion = self.ambient_occlusion.clamp(0.0, 1.0);
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

    #[must_use]
    pub(crate) fn style_values(self) -> [f32; 4] {
        let settings = self.sanitized();
        [
            settings.shadow_strength,
            settings.detail_strength,
            settings.boundary_strength,
            settings.layer_opacity,
        ]
    }

    #[must_use]
    pub(crate) fn lighting_values(self) -> [f32; 4] {
        let settings = self.sanitized();
        [
            settings.sun_azimuth_degrees.to_radians(),
            settings.sun_elevation_degrees.to_radians(),
            settings.ambient_occlusion,
            0.0,
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
        assert_eq!(MapDisplayMode::Tectonics.shader_id(), 5);
        assert_eq!(MapDisplayMode::Hydrology.shader_id(), 6);
        assert_eq!(MapDisplayMode::Climate.shader_id(), 7);
        assert_eq!(MapDisplayMode::Soil.shader_id(), 8);
        assert_eq!(MapDisplayMode::Vegetation.shader_id(), 9);
        assert_eq!(MapDisplayMode::Geology.shader_id(), 10);
        assert_eq!(MapDisplayMode::Resources.shader_id(), 11);
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
        assert!((settings.shadow_strength - 0.62).abs() < f32::EPSILON);
    }

    #[test]
    fn uniform_ranges_are_bounded() {
        let settings = MapDisplaySettings {
            contour_interval_m: 0.01,
            relief_strength: 20.0,
            shadow_strength: -2.0,
            detail_strength: 8.0,
            boundary_strength: 3.0,
            layer_opacity: -1.0,
            ..MapDisplaySettings::default()
        }
        .sanitized();

        assert!((settings.contour_interval_m - 1.0).abs() < f32::EPSILON);
        assert!((settings.relief_strength - 2.0).abs() < f32::EPSILON);
        assert!(settings.shadow_strength.abs() < f32::EPSILON);
        assert!((settings.detail_strength - 1.5).abs() < f32::EPSILON);
        assert!((settings.boundary_strength - 1.0).abs() < f32::EPSILON);
        assert!(settings.layer_opacity.abs() < f32::EPSILON);
    }
}
