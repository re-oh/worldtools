use serde::{Deserialize, Serialize};

use crate::tile::TileId;

const SEED_CONTEXT: &str = "worldtools.world-seed.v1";

/// Stable, portable root seed. It never depends on Rust's randomized `Hash`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldSeed(pub u64);

impl WorldSeed {
    #[must_use]
    pub fn key(self, domain: &str) -> SeedKey {
        let mut hasher = blake3::Hasher::new_derive_key(SEED_CONTEXT);
        hasher.update(&self.0.to_le_bytes());
        update_length_prefixed(&mut hasher, domain.as_bytes());
        SeedKey(*hasher.finalize().as_bytes())
    }

    #[must_use]
    pub fn tile_key(self, domain: &str, tile: TileId) -> SeedKey {
        let mut hasher = blake3::Hasher::new_derive_key(SEED_CONTEXT);
        hasher.update(&self.0.to_le_bytes());
        update_length_prefixed(&mut hasher, domain.as_bytes());
        hasher.update(&tile.stable_bytes());
        SeedKey(*hasher.finalize().as_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SeedKey(pub [u8; 32]);

impl SeedKey {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    #[must_use]
    pub fn u64(self) -> u64 {
        u64::from_le_bytes([
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7],
        ])
    }

    #[must_use]
    pub fn noise_seed(self) -> i32 {
        i32::from_le_bytes([self.0[0], self.0[1], self.0[2], self.0[3]])
    }
}

fn update_length_prefixed(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::CubeFace;

    #[test]
    fn keys_are_deterministic_and_domain_separated() {
        let seed = WorldSeed(42);
        assert_eq!(seed.key("terrain"), seed.key("terrain"));
        assert_ne!(seed.key("terrain"), seed.key("climate"));

        let tile = TileId::new(CubeFace::PositiveX, 4, 3, 9).unwrap();
        assert_ne!(seed.key("terrain"), seed.tile_key("terrain", tile));
    }
}
