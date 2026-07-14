use std::collections::HashSet;

use glam::DVec3;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thiserror::Error;

use crate::{
    geo::{GeoPoint, angular_distance, angular_distance_to_arc},
    tile::{MAX_TILE_LEVEL, ROOT_TILES, TileId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EditId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct EditRevision(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BrushFalloff {
    Hard,
    Linear,
    Smooth,
}

impl BrushFalloff {
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Normalized brush weights intentionally use f32 storage.
    pub fn weight(self, normalized_distance: f64) -> f32 {
        let t = (1.0 - normalized_distance).clamp(0.0, 1.0) as f32;
        match self {
            Self::Hard => f32::from(t > 0.0),
            Self::Linear => t,
            Self::Smooth => t * t * (3.0 - 2.0 * t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EditOperation {
    AddElevation { amount_m: f32 },
    SetElevation { elevation_m: f32 },
}

impl EditOperation {
    fn apply(self, elevation_m: f32, weight: f32) -> f32 {
        match self {
            Self::AddElevation { amount_m } => elevation_m + amount_m * weight,
            Self::SetElevation {
                elevation_m: target,
            } => elevation_m + (target - elevation_m) * weight,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum TerrainEditError {
    #[error("a terrain edit requires at least one path point")]
    EmptyPath,
    #[error("brush radius must be finite and greater than zero")]
    InvalidRadius,
    #[error("brush opacity must be finite and in the inclusive range 0..=1")]
    InvalidOpacity,
    #[error("operation contains a non-finite elevation")]
    InvalidOperation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerrainEdit {
    pub id: EditId,
    pub path: SmallVec<[GeoPoint; 8]>,
    pub radius_m: f64,
    pub opacity: f32,
    pub falloff: BrushFalloff,
    pub operation: EditOperation,
}

impl TerrainEdit {
    /// Creates a validated, resolution-independent geodesic brush stroke.
    ///
    /// # Errors
    /// Returns an error for an empty path, invalid radius or opacity, or a
    /// non-finite operation value.
    pub fn new(
        id: EditId,
        path: impl IntoIterator<Item = GeoPoint>,
        radius_m: f64,
        opacity: f32,
        falloff: BrushFalloff,
        operation: EditOperation,
    ) -> Result<Self, TerrainEditError> {
        let path = path.into_iter().collect::<SmallVec<_>>();
        if path.is_empty() {
            return Err(TerrainEditError::EmptyPath);
        }
        if !radius_m.is_finite() || radius_m <= 0.0 {
            return Err(TerrainEditError::InvalidRadius);
        }
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(TerrainEditError::InvalidOpacity);
        }
        let operation_is_finite = match operation {
            EditOperation::AddElevation { amount_m } => amount_m.is_finite(),
            EditOperation::SetElevation { elevation_m } => elevation_m.is_finite(),
        };
        if !operation_is_finite {
            return Err(TerrainEditError::InvalidOperation);
        }
        Ok(Self {
            id,
            path,
            radius_m,
            opacity,
            falloff,
            operation,
        })
    }

    #[must_use]
    pub fn influence_at(&self, direction: DVec3, planet_radius_m: f64) -> f32 {
        let angular_radius = self.angular_radius(planet_radius_m);
        if angular_radius <= 0.0 {
            return 0.0;
        }
        let distance = self.angular_distance_to_path(direction);
        self.falloff.weight(distance / angular_radius) * self.opacity
    }

    #[must_use]
    pub fn might_affect(&self, tile: TileId, planet_radius_m: f64) -> bool {
        let (center, tile_radius) = tile.bounding_cap();
        self.might_affect_cap(center, tile_radius, planet_radius_m)
    }

    /// Tests a conservative spherical cap against this stroke. Projection and
    /// renderer caches use this without depending on the simulation tile grid.
    #[must_use]
    pub fn might_affect_cap(
        &self,
        center: DVec3,
        angular_radius: f64,
        planet_radius_m: f64,
    ) -> bool {
        self.angular_distance_to_path(center)
            <= self.angular_radius(planet_radius_m) + angular_radius.max(0.0)
    }

    /// Computes a conservative tile cover by descending intersecting roots.
    ///
    /// # Errors
    /// Returns an error for an unsupported level, invalid planet radius, or
    /// when the conservative cover would exceed `limit`.
    pub fn affected_tiles(
        &self,
        level: u8,
        planet_radius_m: f64,
        limit: usize,
    ) -> Result<Vec<TileId>, EditCoverError> {
        if level > MAX_TILE_LEVEL {
            return Err(EditCoverError::LevelTooHigh(level));
        }
        if !planet_radius_m.is_finite() || planet_radius_m <= 0.0 {
            return Err(EditCoverError::InvalidPlanetRadius);
        }

        let mut tiles = Vec::new();
        for root in ROOT_TILES {
            self.collect_affected(root, level, planet_radius_m, limit, &mut tiles)?;
        }
        Ok(tiles)
    }

    fn collect_affected(
        &self,
        tile: TileId,
        target_level: u8,
        planet_radius_m: f64,
        limit: usize,
        output: &mut Vec<TileId>,
    ) -> Result<(), EditCoverError> {
        if !self.might_affect(tile, planet_radius_m) {
            return Ok(());
        }
        if tile.level == target_level {
            if output.len() >= limit {
                return Err(EditCoverError::LimitExceeded { limit });
            }
            output.push(tile);
            return Ok(());
        }
        for child in tile
            .children()
            .expect("target level is below the tile level limit")
        {
            self.collect_affected(child, target_level, planet_radius_m, limit, output)?;
        }
        Ok(())
    }

    fn angular_radius(&self, planet_radius_m: f64) -> f64 {
        (self.radius_m / planet_radius_m).clamp(0.0, std::f64::consts::PI)
    }

    fn angular_distance_to_path(&self, direction: DVec3) -> f64 {
        let mut points = self.path.iter().map(|point| point.direction());
        let Some(mut previous) = points.next() else {
            return f64::INFINITY;
        };
        let mut minimum = angular_distance(direction, previous);
        for current in points {
            minimum = minimum.min(angular_distance_to_arc(direction, previous, current));
            previous = current;
        }
        minimum
    }

    fn update_hash(&self, hasher: &mut blake3::Hasher) {
        hasher.update(&self.id.0.to_le_bytes());
        hasher.update(&self.radius_m.to_bits().to_le_bytes());
        hasher.update(&self.opacity.to_bits().to_le_bytes());
        hasher.update(&[self.falloff as u8]);
        match self.operation {
            EditOperation::AddElevation { amount_m } => {
                hasher.update(&[0]);
                hasher.update(&amount_m.to_bits().to_le_bytes());
            }
            EditOperation::SetElevation { elevation_m } => {
                hasher.update(&[1]);
                hasher.update(&elevation_m.to_bits().to_le_bytes());
            }
        }
        hasher.update(&(self.path.len() as u64).to_le_bytes());
        for point in &self.path {
            hasher.update(&point.latitude.to_bits().to_le_bytes());
            hasher.update(&point.longitude.to_bits().to_le_bytes());
        }
    }

    pub(crate) fn apply_elevation(
        &self,
        direction: DVec3,
        elevation_m: f32,
        planet_radius_m: f64,
    ) -> f32 {
        self.operation
            .apply(elevation_m, self.influence_at(direction, planet_radius_m))
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum EditCoverError {
    #[error("requested edit cover level {0} exceeds {MAX_TILE_LEVEL}")]
    LevelTooHigh(u8),
    #[error("planet radius must be finite and greater than zero")]
    InvalidPlanetRadius,
    #[error("edit cover exceeded its tile limit of {limit}")]
    LimitExceeded { limit: usize },
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum EditJournalError {
    #[error("edit id {0:?} is already present")]
    DuplicateId(EditId),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditJournal {
    revision: EditRevision,
    next_id: u64,
    edits: Vec<TerrainEdit>,
}

impl EditJournal {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn revision(&self) -> EditRevision {
        self.revision
    }

    #[must_use]
    pub fn edits(&self) -> &[TerrainEdit] {
        &self.edits
    }

    pub fn allocate_id(&mut self) -> EditId {
        let id = EditId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        id
    }

    /// # Errors
    /// Returns [`EditJournalError::DuplicateId`] when the id already exists.
    pub fn insert(&mut self, edit: TerrainEdit) -> Result<EditRevision, EditJournalError> {
        if self.edits.iter().any(|existing| existing.id == edit.id) {
            return Err(EditJournalError::DuplicateId(edit.id));
        }
        self.next_id = self.next_id.max(edit.id.0.saturating_add(1));
        self.edits.push(edit);
        self.advance_revision();
        Ok(self.revision)
    }

    pub fn upsert(&mut self, edit: TerrainEdit) -> EditRevision {
        if let Some(existing) = self
            .edits
            .iter_mut()
            .find(|existing| existing.id == edit.id)
        {
            *existing = edit;
        } else {
            self.next_id = self.next_id.max(edit.id.0.saturating_add(1));
            self.edits.push(edit);
        }
        self.advance_revision();
        self.revision
    }

    pub fn remove(&mut self, id: EditId) -> Option<TerrainEdit> {
        let index = self.edits.iter().position(|edit| edit.id == id)?;
        let edit = self.edits.remove(index);
        self.advance_revision();
        Some(edit)
    }

    pub fn clear(&mut self) {
        if !self.edits.is_empty() {
            self.edits.clear();
            self.advance_revision();
        }
    }

    #[must_use]
    pub fn apply_elevation(
        &self,
        direction: DVec3,
        base_elevation_m: f32,
        planet_radius_m: f64,
    ) -> f32 {
        self.edits.iter().fold(base_elevation_m, |elevation, edit| {
            edit.apply_elevation(direction, elevation, planet_radius_m)
        })
    }

    pub fn edits_affecting(
        &self,
        tile: TileId,
        planet_radius_m: f64,
    ) -> impl Iterator<Item = &TerrainEdit> {
        self.edits
            .iter()
            .filter(move |edit| edit.might_affect(tile, planet_radius_m))
    }

    /// Hashes only edits whose conservative bounds intersect the tile. This
    /// lets unrelated strokes keep existing cache entries valid.
    #[must_use]
    pub fn tile_fingerprint(&self, tile: TileId, planet_radius_m: f64) -> u64 {
        let mut hasher = blake3::Hasher::new_derive_key("worldtools.tile-edits.v1");
        for edit in self
            .edits
            .iter()
            .filter(|edit| edit.might_affect(tile, planet_radius_m))
        {
            edit.update_hash(&mut hasher);
        }
        let hash = hasher.finalize();
        let bytes = hash.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    }

    /// Unions conservative covers for selected edits.
    ///
    /// # Errors
    /// Returns an error for invalid cover parameters or when the unique tile
    /// result would exceed `limit`.
    pub fn affected_tiles(
        &self,
        ids: impl IntoIterator<Item = EditId>,
        level: u8,
        planet_radius_m: f64,
        limit: usize,
    ) -> Result<Vec<TileId>, EditCoverError> {
        let ids = ids.into_iter().collect::<HashSet<_>>();
        let mut result = HashSet::new();
        for edit in self.edits.iter().filter(|edit| ids.contains(&edit.id)) {
            for tile in edit.affected_tiles(level, planet_radius_m, limit)? {
                result.insert(tile);
                if result.len() > limit {
                    return Err(EditCoverError::LimitExceeded { limit });
                }
            }
        }
        let mut result = result.into_iter().collect::<Vec<_>>();
        result.sort_unstable_by_key(|tile| (tile.face as u8, tile.level, tile.y, tile.x));
        Ok(result)
    }

    fn advance_revision(&mut self) {
        self.revision.0 = self.revision.0.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::CubeFace;

    const EARTH_RADIUS_M: f64 = 6_371_000.0;

    fn edit(id: u64, point: GeoPoint) -> TerrainEdit {
        TerrainEdit::new(
            EditId(id),
            [point],
            50_000.0,
            1.0,
            BrushFalloff::Smooth,
            EditOperation::AddElevation { amount_m: 100.0 },
        )
        .unwrap()
    }

    #[test]
    fn brush_influence_is_geodesic() {
        let edit = edit(0, GeoPoint::from_degrees(0.0, 179.9));
        let across_dateline = GeoPoint::from_degrees(0.0, -179.9).direction();
        assert!(edit.influence_at(across_dateline, EARTH_RADIUS_M) > 0.0);
    }

    #[test]
    fn hierarchical_cover_contains_the_center_tile() {
        let edit = edit(0, GeoPoint::from_degrees(35.0, 21.0));
        let level = 8;
        let center = TileId::from_direction(edit.path[0].direction(), level).unwrap();
        let affected = edit.affected_tiles(level, EARTH_RADIUS_M, 10_000).unwrap();
        assert!(affected.contains(&center));
        assert!(affected.len() < 100);
    }

    #[test]
    fn tile_fingerprint_ignores_distant_edits() {
        let tile = TileId::new(CubeFace::PositiveX, 6, 32, 32).unwrap();
        let mut journal = EditJournal::new();
        let before = journal.tile_fingerprint(tile, EARTH_RADIUS_M);
        journal
            .insert(edit(4, GeoPoint::from_degrees(0.0, 180.0)))
            .unwrap();
        assert_eq!(before, journal.tile_fingerprint(tile, EARTH_RADIUS_M));
    }
}
