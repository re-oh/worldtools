use serde::{Deserialize, Serialize};
use worldtools_world::{TILE_CELLS, TILE_SAMPLES, TerrainTile, angular_distance};

const EDGE_ALIGNMENT_TOLERANCE_RADIANS: f64 = 1.0e-10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeDirection {
    East,
    South,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileEdge {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct SeamAudit {
    pub compared_samples: usize,
    pub maximum_absolute_error_m: f32,
    pub root_mean_square_error_m: f64,
}

/// Audits two geometrically coincident tile edges.
///
/// `reverse_second` reverses the second edge's sample order. Returns `None`
/// when the edge endpoint directions do not coincide on the cube sphere.
#[must_use]
pub fn audit_tile_seam(
    first: &TerrainTile,
    first_edge: TileEdge,
    second: &TerrainTile,
    second_edge: TileEdge,
    reverse_second: bool,
) -> Option<SeamAudit> {
    let second_start = usize::from(reverse_second) * TILE_CELLS;
    let second_end = TILE_CELLS - second_start;
    let first_start = edge_coordinates(first_edge, 0);
    let first_end = edge_coordinates(first_edge, TILE_CELLS);
    let second_start = edge_coordinates(second_edge, second_start);
    let second_end = edge_coordinates(second_edge, second_end);
    let endpoints_align = angular_distance(
        first.id.sample_direction(first_start.0, first_start.1),
        second.id.sample_direction(second_start.0, second_start.1),
    ) <= EDGE_ALIGNMENT_TOLERANCE_RADIANS
        && angular_distance(
            first.id.sample_direction(first_end.0, first_end.1),
            second.id.sample_direction(second_end.0, second_end.1),
        ) <= EDGE_ALIGNMENT_TOLERANCE_RADIANS;
    if !endpoints_align {
        return None;
    }

    let mut maximum = 0.0_f32;
    let mut squared = 0.0_f64;
    for offset in 0..TILE_SAMPLES {
        let second_offset = if reverse_second {
            TILE_CELLS - offset
        } else {
            offset
        };
        let first_height = edge_height(first, first_edge, offset);
        let second_height = edge_height(second, second_edge, second_offset);
        let error = (first_height - second_height).abs();
        maximum = maximum.max(error);
        squared += f64::from(error) * f64::from(error);
    }

    Some(SeamAudit {
        compared_samples: TILE_SAMPLES,
        maximum_absolute_error_m: maximum,
        root_mean_square_error_m: (squared / 257.0).sqrt(),
    })
}

#[must_use]
pub fn audit_same_face_seam(
    first: &TerrainTile,
    second: &TerrainTile,
    direction: EdgeDirection,
) -> Option<SeamAudit> {
    let aligned = first.id.face == second.id.face
        && first.id.level == second.id.level
        && match direction {
            EdgeDirection::East => first.id.x + 1 == second.id.x && first.id.y == second.id.y,
            EdgeDirection::South => first.id.y + 1 == second.id.y && first.id.x == second.id.x,
        };
    if !aligned {
        return None;
    }

    let (first_edge, second_edge) = match direction {
        EdgeDirection::East => (TileEdge::East, TileEdge::West),
        EdgeDirection::South => (TileEdge::South, TileEdge::North),
    };
    audit_tile_seam(first, first_edge, second, second_edge, false)
}

fn edge_height(tile: &TerrainTile, edge: TileEdge, offset: usize) -> f32 {
    let (x, y) = edge_coordinates(edge, offset);
    tile.interior_sample(x, y)
}

fn edge_coordinates(edge: TileEdge, offset: usize) -> (usize, usize) {
    match edge {
        TileEdge::North => (offset, 0),
        TileEdge::East => (TILE_CELLS, offset),
        TileEdge::South => (offset, TILE_CELLS),
        TileEdge::West => (0, offset),
    }
}

#[cfg(test)]
mod tests {
    use worldtools_world::{CubeFace, TerrainGenerator, TerrainSettings, TileId, WorldSeed};

    use super::*;

    #[test]
    fn cube_face_edge_audit_handles_orientation() {
        let generator = TerrainGenerator::new(WorldSeed(5), TerrainSettings::default());
        let positive_x = generator.generate(TileId::root(CubeFace::PositiveX));
        let negative_z = generator.generate(TileId::root(CubeFace::NegativeZ));
        let audit = audit_tile_seam(
            &positive_x,
            TileEdge::East,
            &negative_z,
            TileEdge::West,
            false,
        )
        .unwrap();
        assert_eq!(audit.compared_samples, TILE_SAMPLES);
        assert_eq!(audit.maximum_absolute_error_m.to_bits(), 0.0_f32.to_bits());
    }

    #[test]
    fn unrelated_edges_are_rejected() {
        let generator = TerrainGenerator::new(WorldSeed(5), TerrainSettings::default());
        let positive_x = generator.generate(TileId::root(CubeFace::PositiveX));
        let negative_x = generator.generate(TileId::root(CubeFace::NegativeX));
        assert!(
            audit_tile_seam(
                &positive_x,
                TileEdge::East,
                &negative_x,
                TileEdge::West,
                false,
            )
            .is_none()
        );
    }
}
