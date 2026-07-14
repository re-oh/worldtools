use std::fmt;

use glam::DVec3;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::geo::{angular_distance, direction_to_face_uv, face_uv_to_direction};

pub const TILE_CELLS: usize = 256;
pub const TILE_SAMPLES: usize = TILE_CELLS + 1;
pub const TILE_APRON: usize = 1;
pub const TILE_STORAGE_SAMPLES: usize = TILE_SAMPLES + 2 * TILE_APRON;
pub const MAX_TILE_LEVEL: u8 = 30;
const TILE_CELLS_I64: i64 = 256;
const TILE_APRON_I64: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum CubeFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl CubeFace {
    pub const ALL: [Self; 6] = [
        Self::PositiveX,
        Self::NegativeX,
        Self::PositiveY,
        Self::NegativeY,
        Self::PositiveZ,
        Self::NegativeZ,
    ];
}

impl fmt::Display for CubeFace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PositiveX => "+X",
            Self::NegativeX => "-X",
            Self::PositiveY => "+Y",
            Self::NegativeY => "-Y",
            Self::PositiveZ => "+Z",
            Self::NegativeZ => "-Z",
        })
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum TileCoordinateError {
    #[error("tile level {0} exceeds the supported maximum of {MAX_TILE_LEVEL}")]
    LevelTooHigh(u8),
    #[error("tile coordinate ({x}, {y}) is outside level {level}, whose axis extent is {extent}")]
    OutOfRange {
        level: u8,
        x: u32,
        y: u32,
        extent: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId {
    pub face: CubeFace,
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

pub const ROOT_TILES: [TileId; 6] = [
    TileId::root(CubeFace::PositiveX),
    TileId::root(CubeFace::NegativeX),
    TileId::root(CubeFace::PositiveY),
    TileId::root(CubeFace::NegativeY),
    TileId::root(CubeFace::PositiveZ),
    TileId::root(CubeFace::NegativeZ),
];

impl TileId {
    #[must_use]
    pub const fn root(face: CubeFace) -> Self {
        Self {
            face,
            level: 0,
            x: 0,
            y: 0,
        }
    }

    /// # Errors
    /// Returns an error when `level` is unsupported or either coordinate lies
    /// outside that level's extent.
    pub fn new(face: CubeFace, level: u8, x: u32, y: u32) -> Result<Self, TileCoordinateError> {
        if level > MAX_TILE_LEVEL {
            return Err(TileCoordinateError::LevelTooHigh(level));
        }
        let extent = 1_u32 << level;
        if x >= extent || y >= extent {
            return Err(TileCoordinateError::OutOfRange {
                level,
                x,
                y,
                extent,
            });
        }
        Ok(Self { face, level, x, y })
    }

    #[must_use]
    pub const fn tiles_per_axis(self) -> u32 {
        1_u32 << self.level
    }

    /// # Errors
    /// Returns an error when `level` exceeds [`MAX_TILE_LEVEL`].
    pub fn from_direction(direction: DVec3, level: u8) -> Result<Self, TileCoordinateError> {
        if level > MAX_TILE_LEVEL {
            return Err(TileCoordinateError::LevelTooHigh(level));
        }
        let (face, u, v) = direction_to_face_uv(direction);
        let extent = 1_u32 << level;
        let x = uv_to_tile_coordinate(u, extent);
        let y = uv_to_tile_coordinate(v, extent);
        Self::new(face, level, x, y)
    }

    #[must_use]
    pub fn parent(self) -> Option<Self> {
        (self.level > 0).then(|| Self {
            face: self.face,
            level: self.level - 1,
            x: self.x / 2,
            y: self.y / 2,
        })
    }

    #[must_use]
    pub fn children(self) -> Option<[Self; 4]> {
        (self.level < MAX_TILE_LEVEL).then(|| {
            let level = self.level + 1;
            let x = self.x * 2;
            let y = self.y * 2;
            [
                Self {
                    face: self.face,
                    level,
                    x,
                    y,
                },
                Self {
                    face: self.face,
                    level,
                    x: x + 1,
                    y,
                },
                Self {
                    face: self.face,
                    level,
                    x,
                    y: y + 1,
                },
                Self {
                    face: self.face,
                    level,
                    x: x + 1,
                    y: y + 1,
                },
            ]
        })
    }

    #[must_use]
    pub fn uv_bounds(self) -> [f64; 4] {
        let extent = f64::from(self.tiles_per_axis());
        let width = 2.0 / extent;
        let u_min = -1.0 + f64::from(self.x) * width;
        let v_min = -1.0 + f64::from(self.y) * width;
        [u_min, v_min, u_min + width, v_min + width]
    }

    /// Returns the sphere direction at an interior grid sample. Both indices
    /// include the shared positive boundary and must be in `0..=256`.
    ///
    /// # Panics
    /// Panics when either sample coordinate is outside `0..=256`.
    #[must_use]
    pub fn sample_direction(self, sample_x: usize, sample_y: usize) -> DVec3 {
        assert!(sample_x < TILE_SAMPLES && sample_y < TILE_SAMPLES);
        self.relative_sample_direction(
            i64::try_from(sample_x).expect("validated sample coordinate fits in i64"),
            i64::try_from(sample_y).expect("validated sample coordinate fits in i64"),
        )
    }

    /// Returns a direction for a grid sample relative to the interior origin.
    /// `-1` and `257` address the one-sample processing apron.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Valid tile numerators have at most 38 significant bits.
    pub fn relative_sample_direction(self, sample_x: i64, sample_y: i64) -> DVec3 {
        let extent = i64::from(self.tiles_per_axis());
        let denominator = extent * TILE_CELLS_I64;
        let global_x = i64::from(self.x) * TILE_CELLS_I64 + sample_x;
        let global_y = i64::from(self.y) * TILE_CELLS_I64 + sample_y;
        let u = -1.0 + 2.0 * (global_x as f64 / denominator as f64);
        let v = -1.0 + 2.0 * (global_y as f64 / denominator as f64);
        face_uv_to_direction(self.face, u, v)
    }

    /// # Panics
    /// Panics when either storage coordinate is outside `0..259`.
    #[must_use]
    pub fn storage_sample_direction(self, storage_x: usize, storage_y: usize) -> DVec3 {
        assert!(storage_x < TILE_STORAGE_SAMPLES && storage_y < TILE_STORAGE_SAMPLES);
        self.relative_sample_direction(
            i64::try_from(storage_x).expect("validated storage coordinate fits in i64")
                - TILE_APRON_I64,
            i64::try_from(storage_y).expect("validated storage coordinate fits in i64")
                - TILE_APRON_I64,
        )
    }

    /// Bounding spherical cap used for conservative hierarchical intersection.
    #[must_use]
    pub fn bounding_cap(self) -> (DVec3, f64) {
        let [u_min, v_min, u_max, v_max] = self.uv_bounds();
        let center = face_uv_to_direction(self.face, (u_min + u_max) * 0.5, (v_min + v_max) * 0.5);
        let boundary_points = [
            (u_min, v_min),
            (u_max, v_min),
            (u_min, v_max),
            (u_max, v_max),
            ((u_min + u_max) * 0.5, v_min),
            ((u_min + u_max) * 0.5, v_max),
            (u_min, (v_min + v_max) * 0.5),
            (u_max, (v_min + v_max) * 0.5),
        ];
        let radius = boundary_points
            .into_iter()
            .map(|(u, v)| angular_distance(center, face_uv_to_direction(self.face, u, v)))
            .fold(0.0_f64, f64::max)
            + 1.0e-12;
        (center, radius)
    }

    #[must_use]
    pub(crate) fn stable_bytes(self) -> [u8; 10] {
        let mut bytes = [0_u8; 10];
        bytes[0] = self.face as u8;
        bytes[1] = self.level;
        bytes[2..6].copy_from_slice(&self.x.to_le_bytes());
        bytes[6..10].copy_from_slice(&self.y.to_le_bytes());
        bytes
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn uv_to_tile_coordinate(coordinate: f64, extent: u32) -> u32 {
    let scaled = ((coordinate + 1.0) * 0.5 * f64::from(extent)).floor();
    scaled.clamp(0.0, f64::from(extent - 1)) as u32
}

#[must_use]
pub const fn storage_index(storage_x: usize, storage_y: usize) -> usize {
    storage_y * TILE_STORAGE_SAMPLES + storage_x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_and_children_are_reciprocal() {
        let parent = TileId::new(CubeFace::PositiveZ, 7, 19, 82).unwrap();
        let children = parent.children().unwrap();
        assert!(
            children
                .into_iter()
                .all(|child| child.parent() == Some(parent))
        );
    }

    #[test]
    fn adjacent_tiles_share_bit_identical_directions() {
        let left = TileId::new(CubeFace::PositiveX, 5, 11, 8).unwrap();
        let right = TileId::new(CubeFace::PositiveX, 5, 12, 8).unwrap();
        for y in 0..TILE_SAMPLES {
            assert_eq!(
                left.sample_direction(TILE_CELLS, y),
                right.sample_direction(0, y)
            );
        }
    }

    #[test]
    fn child_even_samples_match_parent_samples() {
        let parent = TileId::new(CubeFace::NegativeZ, 4, 5, 7).unwrap();
        for child in parent.children().unwrap() {
            let quadrant_x = usize::try_from(child.x & 1).unwrap();
            let quadrant_y = usize::try_from(child.y & 1).unwrap();
            for y in (0..TILE_SAMPLES).step_by(2) {
                for x in (0..TILE_SAMPLES).step_by(2) {
                    let parent_x = quadrant_x * (TILE_CELLS / 2) + x / 2;
                    let parent_y = quadrant_y * (TILE_CELLS / 2) + y / 2;
                    assert_eq!(
                        child.sample_direction(x, y),
                        parent.sample_direction(parent_x, parent_y)
                    );
                }
            }
        }
    }
}
