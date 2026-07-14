use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::tile::CubeFace;

const DIRECTION_EPSILON: f64 = 1.0e-15;

/// A location on the planet in radians.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoPoint {
    #[must_use]
    pub fn from_radians(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude: latitude.clamp(-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2),
            longitude: wrap_longitude(longitude),
        }
    }

    #[must_use]
    pub fn from_degrees(latitude: f64, longitude: f64) -> Self {
        Self::from_radians(latitude.to_radians(), longitude.to_radians())
    }

    #[must_use]
    pub fn direction(self) -> DVec3 {
        let (sin_lat, cos_lat) = self.latitude.sin_cos();
        let (sin_lon, cos_lon) = self.longitude.sin_cos();
        DVec3::new(cos_lat * cos_lon, sin_lat, cos_lat * sin_lon)
    }

    #[must_use]
    pub fn from_direction(direction: DVec3) -> Self {
        let direction = normalized_or_x(direction);
        Self::from_radians(direction.y.asin(), direction.z.atan2(direction.x))
    }
}

/// Converts face-local coordinates in `[-1, 1]` to a unit sphere direction.
/// Values outside the range are accepted for tile apron samples.
#[must_use]
pub fn face_uv_to_direction(face: CubeFace, u: f64, v: f64) -> DVec3 {
    let cube = match face {
        CubeFace::PositiveX => DVec3::new(1.0, v, -u),
        CubeFace::NegativeX => DVec3::new(-1.0, v, u),
        CubeFace::PositiveY => DVec3::new(u, 1.0, -v),
        CubeFace::NegativeY => DVec3::new(u, -1.0, v),
        CubeFace::PositiveZ => DVec3::new(u, v, 1.0),
        CubeFace::NegativeZ => DVec3::new(-u, v, -1.0),
    };
    normalized_or_x(cube)
}

/// Projects a sphere direction onto its dominant cube face.
///
/// Ties use X, then Y, then Z. That deterministic ownership matters on exact
/// face boundaries, where either adjacent face is geometrically valid.
#[must_use]
pub fn direction_to_face_uv(direction: DVec3) -> (CubeFace, f64, f64) {
    let direction = normalized_or_x(direction);
    let absolute = direction.abs();

    if absolute.x >= absolute.y && absolute.x >= absolute.z {
        if direction.x >= 0.0 {
            (
                CubeFace::PositiveX,
                -direction.z / absolute.x,
                direction.y / absolute.x,
            )
        } else {
            (
                CubeFace::NegativeX,
                direction.z / absolute.x,
                direction.y / absolute.x,
            )
        }
    } else if absolute.y >= absolute.z {
        if direction.y >= 0.0 {
            (
                CubeFace::PositiveY,
                direction.x / absolute.y,
                -direction.z / absolute.y,
            )
        } else {
            (
                CubeFace::NegativeY,
                direction.x / absolute.y,
                direction.z / absolute.y,
            )
        }
    } else if direction.z >= 0.0 {
        (
            CubeFace::PositiveZ,
            direction.x / absolute.z,
            direction.y / absolute.z,
        )
    } else {
        (
            CubeFace::NegativeZ,
            -direction.x / absolute.z,
            direction.y / absolute.z,
        )
    }
}

#[must_use]
pub fn angular_distance(a: DVec3, b: DVec3) -> f64 {
    normalized_or_x(a)
        .dot(normalized_or_x(b))
        .clamp(-1.0, 1.0)
        .acos()
}

/// Minimum angular distance from `point` to the minor great-circle arc AB.
#[must_use]
pub(crate) fn angular_distance_to_arc(point: DVec3, a: DVec3, b: DVec3) -> f64 {
    let point = normalized_or_x(point);
    let a = normalized_or_x(a);
    let b = normalized_or_x(b);
    let arc_length = angular_distance(a, b);
    let normal = a.cross(b);

    if normal.length_squared() <= DIRECTION_EPSILON || arc_length >= std::f64::consts::PI - 1.0e-10
    {
        return angular_distance(point, a).min(angular_distance(point, b));
    }

    let normal = normal.normalize();
    let projected = point - normal * point.dot(normal);
    if projected.length_squared() <= DIRECTION_EPSILON {
        return angular_distance(point, a).min(angular_distance(point, b));
    }

    let projected = projected.normalize();
    let opposite = -projected;
    for candidate in [projected, opposite] {
        let split = angular_distance(a, candidate) + angular_distance(candidate, b);
        if split <= arc_length + 1.0e-10 {
            return angular_distance(point, candidate);
        }
    }

    angular_distance(point, a).min(angular_distance(point, b))
}

fn normalized_or_x(direction: DVec3) -> DVec3 {
    if direction.is_finite() && direction.length_squared() > DIRECTION_EPSILON {
        direction.normalize()
    } else {
        DVec3::X
    }
}

fn wrap_longitude(longitude: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    (longitude + std::f64::consts::PI).rem_euclid(tau) - std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longitude_wraps_to_canonical_range() {
        let point = GeoPoint::from_degrees(20.0, 540.0);
        assert!((point.longitude.to_degrees() + 180.0).abs() < 1.0e-10);
    }

    #[test]
    fn face_projection_round_trips_interiors() {
        for face in CubeFace::ALL {
            for (u, v) in [(-0.8, 0.7), (0.0, 0.0), (0.63, -0.41)] {
                let direction = face_uv_to_direction(face, u, v);
                let (actual_face, actual_u, actual_v) = direction_to_face_uv(direction);
                assert_eq!(actual_face, face);
                assert!((actual_u - u).abs() < 1.0e-12);
                assert!((actual_v - v).abs() < 1.0e-12);
            }
        }
    }
}
