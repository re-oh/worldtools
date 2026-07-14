use proptest::prelude::*;
use worldtools_world::{
    CubeFace, TILE_CELLS, TerrainGenerator, TerrainSettings, TileId, WorldSeed,
    direction_to_face_uv, face_uv_to_direction,
};

fn face_strategy() -> impl Strategy<Value = CubeFace> {
    prop_oneof![
        Just(CubeFace::PositiveX),
        Just(CubeFace::NegativeX),
        Just(CubeFace::PositiveY),
        Just(CubeFace::NegativeY),
        Just(CubeFace::PositiveZ),
        Just(CubeFace::NegativeZ),
    ]
}

proptest! {
    #[test]
    fn interior_face_coordinates_round_trip(
        face in face_strategy(),
        u in -0.999_f64..0.999,
        v in -0.999_f64..0.999,
    ) {
        let (actual_face, actual_u, actual_v) =
            direction_to_face_uv(face_uv_to_direction(face, u, v));
        prop_assert_eq!(actual_face, face);
        prop_assert!((actual_u - u).abs() < 1.0e-12);
        prop_assert!((actual_v - v).abs() < 1.0e-12);
    }

    #[test]
    fn same_face_neighbors_share_exact_samples(
        face in face_strategy(),
        level in 1_u8..12,
        raw_x in any::<u32>(),
        raw_y in any::<u32>(),
        sample_y in 0_usize..=TILE_CELLS,
    ) {
        let extent = 1_u32 << level;
        let x = raw_x % (extent - 1);
        let y = raw_y % extent;
        let left = TileId::new(face, level, x, y).unwrap();
        let right = TileId::new(face, level, x + 1, y).unwrap();
        prop_assert_eq!(
            left.sample_direction(TILE_CELLS, sample_y),
            right.sample_direction(0, sample_y),
        );
    }

    #[test]
    fn parent_samples_are_present_in_children(
        face in face_strategy(),
        level in 0_u8..11,
        raw_x in any::<u32>(),
        raw_y in any::<u32>(),
        child_index in 0_usize..4,
        sample_x in 0_usize..=128,
        sample_y in 0_usize..=128,
    ) {
        let extent = 1_u32 << level;
        let parent = TileId::new(face, level, raw_x % extent, raw_y % extent).unwrap();
        let child = parent.children().unwrap()[child_index];
        let quadrant_x = (child.x & 1) as usize;
        let quadrant_y = (child.y & 1) as usize;
        prop_assert_eq!(
            parent.sample_direction(
                quadrant_x * 128 + sample_x,
                quadrant_y * 128 + sample_y,
            ),
            child.sample_direction(sample_x * 2, sample_y * 2),
        );
    }

    #[test]
    fn continuous_generation_is_deterministic(
        seed in any::<u64>(),
        face in face_strategy(),
        u in -1.0_f64..1.0,
        v in -1.0_f64..1.0,
    ) {
        let generator = TerrainGenerator::new(WorldSeed(seed), TerrainSettings::default());
        let direction = face_uv_to_direction(face, u, v);
        prop_assert_eq!(
            generator.sample_elevation_m(direction).to_bits(),
            generator.sample_elevation_m(direction).to_bits(),
        );
    }
}

#[test]
fn cube_face_boundary_is_seamless() {
    let generator = TerrainGenerator::new(WorldSeed(0x00C0_FFEE), TerrainSettings::default());
    let positive_x = TileId::root(CubeFace::PositiveX);
    let negative_z = TileId::root(CubeFace::NegativeZ);

    for sample_y in 0..=TILE_CELLS {
        let x_direction = positive_x.sample_direction(TILE_CELLS, sample_y);
        let z_direction = negative_z.sample_direction(0, sample_y);
        assert_eq!(x_direction, z_direction);
        assert_eq!(
            generator.sample_elevation_m(x_direction).to_bits(),
            generator.sample_elevation_m(z_direction).to_bits(),
        );
    }
}

#[test]
fn every_cube_edge_has_an_identical_partner() {
    use CubeFace::{NegativeX, NegativeY, NegativeZ, PositiveX, PositiveY, PositiveZ};

    // (face A, fixed U/V selector, face B, fixed U/V selector, reverse parameter)
    let edge_pairs = [
        (
            PositiveX,
            [1.0, f64::NAN],
            NegativeZ,
            [-1.0, f64::NAN],
            false,
        ),
        (
            PositiveX,
            [-1.0, f64::NAN],
            PositiveZ,
            [1.0, f64::NAN],
            false,
        ),
        (
            NegativeX,
            [1.0, f64::NAN],
            PositiveZ,
            [-1.0, f64::NAN],
            false,
        ),
        (
            NegativeX,
            [-1.0, f64::NAN],
            NegativeZ,
            [1.0, f64::NAN],
            false,
        ),
        (
            PositiveY,
            [f64::NAN, -1.0],
            PositiveZ,
            [f64::NAN, 1.0],
            false,
        ),
        (PositiveY, [f64::NAN, 1.0], NegativeZ, [f64::NAN, 1.0], true),
        (
            NegativeY,
            [f64::NAN, 1.0],
            PositiveZ,
            [f64::NAN, -1.0],
            false,
        ),
        (
            NegativeY,
            [f64::NAN, -1.0],
            NegativeZ,
            [f64::NAN, -1.0],
            true,
        ),
        (
            PositiveX,
            [f64::NAN, 1.0],
            PositiveY,
            [1.0, f64::NAN],
            false,
        ),
        (
            PositiveX,
            [f64::NAN, -1.0],
            NegativeY,
            [1.0, f64::NAN],
            true,
        ),
        (
            NegativeX,
            [f64::NAN, 1.0],
            PositiveY,
            [-1.0, f64::NAN],
            true,
        ),
        (
            NegativeX,
            [f64::NAN, -1.0],
            NegativeY,
            [-1.0, f64::NAN],
            false,
        ),
    ];

    for (face_a, edge_a, face_b, edge_b, reverse) in edge_pairs {
        for step in 0..=64 {
            let parameter = -1.0 + 2.0 * f64::from(step) / 64.0;
            let other_parameter = if reverse { -parameter } else { parameter };
            let [u_a, v_a] = edge_coordinates(edge_a, parameter);
            let [u_b, v_b] = edge_coordinates(edge_b, other_parameter);
            assert_eq!(
                face_uv_to_direction(face_a, u_a, v_a),
                face_uv_to_direction(face_b, u_b, v_b),
                "edge mismatch for {face_a:?} and {face_b:?} at {parameter}",
            );
        }
    }
}

fn edge_coordinates(edge: [f64; 2], parameter: f64) -> [f64; 2] {
    [
        if edge[0].is_nan() { parameter } else { edge[0] },
        if edge[1].is_nan() { parameter } else { edge[1] },
    ]
}
