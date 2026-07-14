#[derive(Debug, Clone, Copy)]
pub(crate) struct Vec3 {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl Vec3 {
    pub(crate) const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub(crate) fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub(crate) fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub(crate) fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub(crate) fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub(crate) fn normalized(self) -> Self {
        let length = self.dot(self).sqrt();
        if length > 1.0e-12 {
            self.scale(1.0 / length)
        } else {
            Self::new(1.0, 0.0, 0.0)
        }
    }

    pub(crate) fn rotate_about(self, axis: Self, angle: f64) -> Self {
        let axis = axis.normalized();
        let (sin, cos) = angle.sin_cos();
        self.scale(cos)
            .add(axis.cross(self).scale(sin))
            .add(axis.scale(axis.dot(self) * (1.0 - cos)))
            .normalized()
    }
}

pub(crate) fn direction(latitude: f64, longitude: f64) -> Vec3 {
    let (sin_lat, cos_lat) = latitude.sin_cos();
    let (sin_lon, cos_lon) = longitude.sin_cos();
    Vec3::new(cos_lat * cos_lon, sin_lat, cos_lat * sin_lon)
}

pub(crate) fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let amount = ((value - low) / (high - low)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}
