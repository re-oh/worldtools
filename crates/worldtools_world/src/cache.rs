use std::sync::Arc;

use moka::sync::Cache;
use serde::{Deserialize, Serialize};

use crate::{seed::WorldSeed, terrain::TerrainTile, tile::TileId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileCacheKey {
    pub world_seed: WorldSeed,
    pub terrain_fingerprint: [u8; 32],
    pub tile: TileId,
}

impl TileCacheKey {
    #[must_use]
    pub const fn new(world_seed: WorldSeed, terrain_fingerprint: [u8; 32], tile: TileId) -> Self {
        Self {
            world_seed,
            terrain_fingerprint,
            tile,
        }
    }
}

#[derive(Clone)]
pub struct TerrainTileCache {
    inner: Cache<TileCacheKey, Arc<TerrainTile>>,
}

impl TerrainTileCache {
    #[must_use]
    pub fn with_capacity_bytes(capacity_bytes: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(capacity_bytes)
            .weigher(|_key, value: &Arc<TerrainTile>| {
                u32::try_from(value.byte_len()).unwrap_or(u32::MAX)
            })
            .build();
        Self { inner }
    }

    #[must_use]
    pub fn get(&self, key: &TileCacheKey) -> Option<Arc<TerrainTile>> {
        self.inner.get(key)
    }

    pub fn insert(&self, key: TileCacheKey, tile: Arc<TerrainTile>) {
        self.inner.insert(key, tile);
    }

    pub fn invalidate(&self, key: &TileCacheKey) {
        self.inner.invalidate(key);
    }

    pub fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }

    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    #[must_use]
    pub fn weighted_size(&self) -> u64 {
        self.inner.weighted_size()
    }

    pub fn synchronize(&self) {
        self.inner.run_pending_tasks();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        terrain::{TerrainGenerator, TerrainSettings},
        tile::CubeFace,
    };

    #[test]
    fn cache_stores_and_invalidates_tiles() {
        let seed = WorldSeed(17);
        let settings = TerrainSettings::default();
        let id = TileId::root(CubeFace::PositiveX);
        let key = TileCacheKey::new(seed, settings.fingerprint(), id);
        let tile = Arc::new(TerrainGenerator::new(seed, settings).generate(id));
        let cache = TerrainTileCache::with_capacity_bytes(2 * tile.byte_len() as u64);

        cache.insert(key, Arc::clone(&tile));
        cache.synchronize();
        assert!(cache.get(&key).is_some());
        cache.invalidate(&key);
        cache.synchronize();
        assert!(cache.get(&key).is_none());
    }
}
