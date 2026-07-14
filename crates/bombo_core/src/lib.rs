mod hash;
mod kernels;

pub use hash::{bombo_random_at, bombo_seed_hash};
pub use kernels::{bombo_affine_clamp_f32, bombo_alloc_f32, bombo_free_f32, bombo_max_f32};

#[unsafe(no_mangle)]
pub extern "C" fn bombo_schema_version() -> u32 {
    300
}

#[unsafe(no_mangle)]
pub extern "C" fn bombo_grid_cell_count(height: u32, width: u32) -> u32 {
    if height < 64 || width < 128 || height > 512 || width > 1024 || width != height * 2 {
        return 0;
    }
    height.checked_mul(width).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abi_matches_world_version() {
        assert_eq!(bombo_schema_version(), 300);
        assert_eq!(bombo_grid_cell_count(64, 128), 8192);
        assert_eq!(bombo_grid_cell_count(64, 256), 0);
    }

    #[test]
    fn random_streams_are_stable_and_distinct() {
        assert_eq!(bombo_random_at(42, 3, 9, 1), bombo_random_at(42, 3, 9, 1));
        assert_ne!(bombo_random_at(42, 3, 9, 1), bombo_random_at(42, 4, 9, 1));
    }
}
