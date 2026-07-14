use worldtools_world::{CubeFace, TileId};

#[must_use]
pub fn all_tiles(level: u8) -> Vec<TileId> {
    let extent = 1_u32 << level;
    CubeFace::ALL
        .into_iter()
        .flat_map(|face| {
            (0..extent).flat_map(move |y| {
                (0..extent).map(move |x| {
                    TileId::new(face, level, x, y)
                        .expect("coordinates enumerated inside the level extent are valid")
                })
            })
        })
        .collect()
}

#[must_use]
pub const fn tiles_per_face(level: u8) -> usize {
    let extent = 1_usize << level;
    extent * extent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_enumeration_covers_all_faces() {
        let tiles = all_tiles(2);
        assert_eq!(tiles.len(), 6 * 16);
        assert!(
            CubeFace::ALL.into_iter().all(|face| tiles
                .iter()
                .filter(|tile| tile.face == face)
                .count()
                == 16)
        );
    }
}
