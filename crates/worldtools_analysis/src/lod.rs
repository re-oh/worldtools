use serde::{Deserialize, Serialize};
use worldtools_world::{TILE_CELLS, TILE_SAMPLES, TerrainTile};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct LodAudit {
    pub compared_samples: usize,
    pub maximum_absolute_error_m: f32,
    pub root_mean_square_error_m: f64,
}

#[must_use]
pub fn audit_child_consistency(parent: &TerrainTile, child: &TerrainTile) -> Option<LodAudit> {
    if child.id.parent() != Some(parent.id) {
        return None;
    }
    let quadrant_x = usize::from(child.id.x & 1 != 0);
    let quadrant_y = usize::from(child.id.y & 1 != 0);
    let mut maximum = 0.0_f32;
    let mut squared = 0.0_f64;
    let mut count = 0_usize;

    for child_y in (0..TILE_SAMPLES).step_by(2) {
        for child_x in (0..TILE_SAMPLES).step_by(2) {
            let parent_x = quadrant_x * (TILE_CELLS / 2) + child_x / 2;
            let parent_y = quadrant_y * (TILE_CELLS / 2) + child_y / 2;
            let error = (child.interior_sample(child_x, child_y)
                - parent.interior_sample(parent_x, parent_y))
            .abs();
            maximum = maximum.max(error);
            squared += f64::from(error) * f64::from(error);
            count += 1;
        }
    }

    Some(LodAudit {
        compared_samples: count,
        maximum_absolute_error_m: maximum,
        root_mean_square_error_m: (squared / count_as_f64(count)).sqrt(),
    })
}

#[allow(clippy::cast_precision_loss)]
fn count_as_f64(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    use worldtools_world::{CubeFace, TerrainGenerator, TerrainSettings, TileId, WorldSeed};

    use super::*;

    #[test]
    fn generated_parent_child_samples_are_exact() {
        let generator = TerrainGenerator::new(WorldSeed(11), TerrainSettings::default());
        let parent = generator.generate(TileId::root(CubeFace::PositiveY));
        let child = generator.generate(parent.id.children().unwrap()[2]);
        let audit = audit_child_consistency(&parent, &child).unwrap();
        assert_eq!(audit.maximum_absolute_error_m.to_bits(), 0.0_f32.to_bits());
        assert!(audit.compared_samples > 0);
    }
}
