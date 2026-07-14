use worldtools_analysis::TileEdge;
use worldtools_world::{CubeFace, TileId};

#[derive(Clone, Copy)]
pub struct CubeEdgeRelation {
    pub first_face: CubeFace,
    pub first_edge: TileEdge,
    pub second_face: CubeFace,
    pub second_edge: TileEdge,
    pub reversed: bool,
}

use CubeFace::{NegativeX, NegativeY, NegativeZ, PositiveX, PositiveY, PositiveZ};
use TileEdge::{East, North, South, West};

pub const CUBE_EDGE_RELATIONS: [CubeEdgeRelation; 12] = [
    CubeEdgeRelation {
        first_face: PositiveX,
        first_edge: West,
        second_face: PositiveZ,
        second_edge: East,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: PositiveX,
        first_edge: East,
        second_face: NegativeZ,
        second_edge: West,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: NegativeX,
        first_edge: West,
        second_face: NegativeZ,
        second_edge: East,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: NegativeX,
        first_edge: East,
        second_face: PositiveZ,
        second_edge: West,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: PositiveY,
        first_edge: North,
        second_face: PositiveZ,
        second_edge: South,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: PositiveY,
        first_edge: South,
        second_face: NegativeZ,
        second_edge: South,
        reversed: true,
    },
    CubeEdgeRelation {
        first_face: NegativeY,
        first_edge: South,
        second_face: PositiveZ,
        second_edge: North,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: NegativeY,
        first_edge: North,
        second_face: NegativeZ,
        second_edge: North,
        reversed: true,
    },
    CubeEdgeRelation {
        first_face: PositiveX,
        first_edge: South,
        second_face: PositiveY,
        second_edge: East,
        reversed: false,
    },
    CubeEdgeRelation {
        first_face: PositiveX,
        first_edge: North,
        second_face: NegativeY,
        second_edge: East,
        reversed: true,
    },
    CubeEdgeRelation {
        first_face: NegativeX,
        first_edge: South,
        second_face: PositiveY,
        second_edge: West,
        reversed: true,
    },
    CubeEdgeRelation {
        first_face: NegativeX,
        first_edge: North,
        second_face: NegativeY,
        second_edge: West,
        reversed: false,
    },
];

#[must_use]
pub fn edge_tile(face: CubeFace, edge: TileEdge, level: u8, offset: u32) -> TileId {
    let extent = 1_u32 << level;
    let (x, y) = match edge {
        North => (offset, 0),
        East => (extent - 1, offset),
        South => (offset, extent - 1),
        West => (0, offset),
    };
    TileId::new(face, level, x, y).expect("cube edge coordinates are inside the level extent")
}

#[cfg(test)]
mod tests {
    use worldtools_analysis::audit_tile_seam;
    use worldtools_world::{TerrainGenerator, TerrainSettings, WorldSeed};

    use super::*;

    #[test]
    fn every_declared_root_edge_is_geometrically_aligned() {
        let generator = TerrainGenerator::new(WorldSeed(31), TerrainSettings::default());
        for relation in CUBE_EDGE_RELATIONS {
            let first =
                generator.generate(edge_tile(relation.first_face, relation.first_edge, 0, 0));
            let second =
                generator.generate(edge_tile(relation.second_face, relation.second_edge, 0, 0));
            assert!(
                audit_tile_seam(
                    &first,
                    relation.first_edge,
                    &second,
                    relation.second_edge,
                    relation.reversed,
                )
                .is_some(),
                "misaligned cube edge between {:?} and {:?}",
                relation.first_face,
                relation.second_face,
            );
        }
    }
}
