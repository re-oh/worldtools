//! Engine-independent world geometry, generation, editing, and tile storage.
//!
//! The crate deliberately knows nothing about Bevy or a particular renderer.
//! Cube-sphere tiles and geodesic edits are the stable contract shared by the
//! desktop application, analysis tools, and future simulation stages.

pub mod cache;
pub mod edit;
pub mod geo;
pub mod seed;
pub mod terrain;
pub mod tile;

/// Common world-building types without cache or editing implementation details.
pub mod prelude {
    pub use crate::{
        CubeFace, GeoPoint, TerrainGenerator, TerrainSettings, TerrainTile, TileId, WorldSeed,
    };
}

pub use cache::{TerrainTileCache, TileCacheKey};
pub use edit::{
    BrushFalloff, EditCoverError, EditId, EditJournal, EditJournalError, EditOperation,
    EditRevision, TerrainEdit, TerrainEditError,
};
pub use geo::{GeoPoint, angular_distance, direction_to_face_uv, face_uv_to_direction};
pub use seed::{SeedKey, WorldSeed};
pub use terrain::{TerrainGenerator, TerrainSettings, TerrainTile, TerrainTileStats};
pub use tile::{
    CubeFace, MAX_TILE_LEVEL, ROOT_TILES, TILE_APRON, TILE_CELLS, TILE_SAMPLES,
    TILE_STORAGE_SAMPLES, TileCoordinateError, TileId,
};
