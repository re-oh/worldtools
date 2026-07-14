use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MapReadout {
    pub cursor_degrees: Option<[f64; 2]>,
    pub meters_per_pixel: f64,
    pub lod: u8,
}

impl Default for MapReadout {
    fn default() -> Self {
        Self {
            cursor_degrees: None,
            meters_per_pixel: 0.0,
            lod: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ViewportRect {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl ViewportRect {
    #[must_use]
    pub fn width(self) -> f32 {
        (self.max[0] - self.min[0]).max(0.0)
    }

    #[must_use]
    pub fn height(self) -> f32 {
        (self.max[1] - self.min[1]).max(0.0)
    }

    #[must_use]
    pub fn physical(self, pixels_per_point: f32) -> Self {
        Self {
            min: [
                (self.min[0] * pixels_per_point).round(),
                (self.min[1] * pixels_per_point).round(),
            ],
            max: [
                (self.max[0] * pixels_per_point).round(),
                (self.max[1] * pixels_per_point).round(),
            ],
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MapViewport {
    pub logical: ViewportRect,
    pub physical: ViewportRect,
    pub pixels_per_point: f32,
    pub hovered: bool,
    pub input_blocked: bool,
    pub frame: u64,
}

impl MapViewport {
    /// Converts the pixel-aligned egui viewport into Bevy window coordinates.
    ///
    /// Egui lays out in points, while Bevy window input and 2D transforms use
    /// logical pixels. Going through physical pixels also accounts for egui's
    /// independent UI zoom factor.
    #[must_use]
    pub fn window_logical(self, native_pixels_per_point: f32) -> ViewportRect {
        let scale = if native_pixels_per_point.is_finite() && native_pixels_per_point > 0.0 {
            native_pixels_per_point
        } else {
            1.0
        };
        ViewportRect {
            min: [self.physical.min[0] / scale, self.physical.min[1] / scale],
            max: [self.physical.max[0] / scale, self.physical.max[1] / scale],
        }
    }
}

impl Default for MapViewport {
    fn default() -> Self {
        Self {
            logical: ViewportRect::default(),
            physical: ViewportRect::default(),
            pixels_per_point: 1.0,
            hovered: false,
            input_blocked: true,
            frame: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_physical_coordinates_are_pixel_aligned() {
        let logical = ViewportRect {
            min: [40.25, 20.5],
            max: [100.75, 50.25],
        };
        let physical = logical.physical(1.5);

        for (actual, expected) in physical.min.into_iter().zip([60.0, 31.0]) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
        for (actual, expected) in physical.max.into_iter().zip([151.0, 75.0]) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn window_coordinates_account_for_independent_ui_zoom() {
        let logical = ViewportRect {
            min: [40.0, 20.0],
            max: [100.0, 60.0],
        };
        let viewport = MapViewport {
            logical,
            physical: logical.physical(3.0),
            pixels_per_point: 3.0,
            ..MapViewport::default()
        };

        let window_logical = viewport.window_logical(2.0);
        for (actual, expected) in window_logical.min.into_iter().zip([60.0, 30.0]) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
        for (actual, expected) in window_logical.max.into_iter().zip([150.0, 90.0]) {
            assert!((actual - expected).abs() < f32::EPSILON);
        }
    }
}
