use worldtools_world::GeoPoint;

/// Cell-centered global equirectangular simulation grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtlasGrid {
    width: usize,
    height: usize,
}

impl AtlasGrid {
    /// # Panics
    /// Panics only on platforms whose pointer width cannot represent a `u32`.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: usize::try_from(width.max(1)).expect("atlas width fits usize"),
            height: usize::try_from(height.max(1)).expect("atlas height fits usize"),
        }
    }

    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> usize {
        self.height
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.width * self.height
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub const fn index(self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[must_use]
    pub const fn coordinates(self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Dimensions are capped far below f64 precision.
    pub fn point(self, index: usize) -> GeoPoint {
        let (x, y) = self.coordinates(index);
        let longitude =
            ((x as f64 + 0.5) / self.width as f64) * std::f64::consts::TAU - std::f64::consts::PI;
        let latitude = std::f64::consts::FRAC_PI_2
            - ((y as f64 + 0.5) / self.height as f64) * std::f64::consts::PI;
        GeoPoint::from_radians(latitude, longitude)
    }

    /// # Panics
    /// Panics only when atlas dimensions cannot be represented as `isize`.
    #[must_use]
    pub fn wrapped_index(self, x: isize, y: isize) -> usize {
        let width = isize::try_from(self.width).expect("atlas width fits isize");
        let height = isize::try_from(self.height).expect("atlas height fits isize");
        let wrapped_x = x.rem_euclid(width);
        let clamped_y = y.clamp(0, height - 1);
        self.index(
            usize::try_from(wrapped_x).expect("wrapped x is non-negative"),
            usize::try_from(clamped_y).expect("clamped y is non-negative"),
        )
    }

    #[must_use]
    pub fn nearest_index(self, point: GeoPoint) -> usize {
        let (x, y, tx, ty) = self.sample_coordinates(point);
        self.wrapped_index(x + isize::from(tx >= 0.5), y + isize::from(ty >= 0.5))
    }

    /// # Panics
    /// Panics when `values` does not contain exactly one value per grid cell.
    #[must_use]
    pub fn sample_scalar(self, values: &[f32], point: GeoPoint) -> f32 {
        assert_eq!(values.len(), self.len());
        let (x0, y0, tx, ty) = self.sample_coordinates(point);
        let a = values[self.wrapped_index(x0, y0)];
        let b = values[self.wrapped_index(x0 + 1, y0)];
        let c = values[self.wrapped_index(x0, y0 + 1)];
        let d = values[self.wrapped_index(x0 + 1, y0 + 1)];
        lerp(lerp(a, b, tx), lerp(c, d, tx), ty)
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn cell_metrics_m(self, index: usize, planet_radius_m: f64) -> (f32, f32) {
        let latitude = self.point(index).latitude;
        let east_west = (planet_radius_m * latitude.cos().abs().max(0.02) * std::f64::consts::TAU
            / self.width as f64) as f32;
        let north_south = (planet_radius_m * std::f64::consts::PI / self.height as f64) as f32;
        (east_west, north_south)
    }

    /// # Panics
    /// Panics when `index` is outside this grid or `values` is too short.
    #[must_use]
    pub fn slope(self, values: &[f32], index: usize, planet_radius_m: f64) -> f32 {
        let (x, y) = self.coordinates(index);
        let x = isize::try_from(x).expect("x fits isize");
        let y = isize::try_from(y).expect("y fits isize");
        let (dx, dy) = self.cell_metrics_m(index, planet_radius_m);
        let east = values[self.wrapped_index(x + 1, y)];
        let west = values[self.wrapped_index(x - 1, y)];
        let north = values[self.wrapped_index(x, y - 1)];
        let south = values[self.wrapped_index(x, y + 1)];
        let gx = (east - west) / (2.0 * dx);
        let gy = (south - north) / (2.0 * dy);
        gx.hypot(gy)
    }

    pub(crate) fn cardinal_neighbors(self, index: usize) -> [usize; 4] {
        let (x, y) = self.coordinates(index);
        let x = isize::try_from(x).expect("x fits isize");
        let y = isize::try_from(y).expect("y fits isize");
        [
            self.wrapped_index(x - 1, y),
            self.wrapped_index(x + 1, y),
            self.wrapped_index(x, y - 1),
            self.wrapped_index(x, y + 1),
        ]
    }

    pub(crate) fn neighbors8(self, index: usize) -> [usize; 8] {
        let (x, y) = self.coordinates(index);
        let x = isize::try_from(x).expect("x fits isize");
        let y = isize::try_from(y).expect("y fits isize");
        [
            self.wrapped_index(x - 1, y - 1),
            self.wrapped_index(x, y - 1),
            self.wrapped_index(x + 1, y - 1),
            self.wrapped_index(x - 1, y),
            self.wrapped_index(x + 1, y),
            self.wrapped_index(x - 1, y + 1),
            self.wrapped_index(x, y + 1),
            self.wrapped_index(x + 1, y + 1),
        ]
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn sample_coordinates(self, point: GeoPoint) -> (isize, isize, f32, f32) {
        let x = ((point.longitude + std::f64::consts::PI) / std::f64::consts::TAU)
            * self.width as f64
            - 0.5;
        let y = ((std::f64::consts::FRAC_PI_2 - point.latitude) / std::f64::consts::PI)
            * self.height as f64
            - 0.5;
        let x0 = x.floor();
        let y0 = y.floor();
        (x0 as isize, y0 as isize, (x - x0) as f32, (y - y0) as f32)
    }
}

fn lerp(a: f32, b: f32, amount: f32) -> f32 {
    a + (b - a) * amount
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn longitude_sampling_wraps_without_a_seam() {
        let grid = AtlasGrid::new(16, 8);
        let values = (0..grid.len())
            .map(|index| grid.point(index).longitude.cos() as f32)
            .collect::<Vec<_>>();
        let west = grid.sample_scalar(&values, GeoPoint::from_degrees(0.0, -179.999));
        let east = grid.sample_scalar(&values, GeoPoint::from_degrees(0.0, 179.999));
        assert!((west - east).abs() < 0.001);
    }

    #[test]
    fn cell_centers_round_trip_to_their_categorical_index() {
        let grid = AtlasGrid::new(72, 36);
        for index in 0..grid.len() {
            assert_eq!(grid.nearest_index(grid.point(index)), index);
        }
    }
}
