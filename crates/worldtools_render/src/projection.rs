use std::collections::HashSet;

use bevy::prelude::Vec2;
use worldtools_world::{GeoPoint, angular_distance};

use crate::view::MapView;

pub const MAP_TILE_CELLS: u32 = 256;
const MAP_TILE_CELLS_F32: f32 = 256.0;
pub const MAP_TILE_APRON: u32 = 4;
pub const MAP_TILE_SAMPLES: u32 = MAP_TILE_CELLS + 1 + MAP_TILE_APRON * 2;
pub const MAP_TILE_SAMPLE_COUNT: usize = 70_225;
pub const MAX_MAP_LEVEL: u8 = 17;
const PREFETCH_MARGIN: i64 = 1;
const MAX_VISIBLE_TILES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MapTileId {
    pub level: u8,
    pub x: u32,
    pub y: u32,
}

impl MapTileId {
    #[must_use]
    pub const fn x_extent(self) -> u32 {
        2_u32 << self.level
    }

    #[must_use]
    pub const fn y_extent(self) -> u32 {
        1_u32 << self.level
    }

    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        if self.level == 0 {
            None
        } else {
            Some(Self {
                level: self.level - 1,
                x: self.x / 2,
                y: self.y / 2,
            })
        }
    }

    #[must_use]
    pub fn bounding_cap(self) -> (GeoPoint, f64) {
        let x_extent = f64::from(self.x_extent());
        let y_extent = f64::from(self.y_extent());
        let west = -180.0 + 360.0 * f64::from(self.x) / x_extent;
        let east = -180.0 + 360.0 * f64::from(self.x + 1) / x_extent;
        let north = 90.0 - 180.0 * f64::from(self.y) / y_extent;
        let south = 90.0 - 180.0 * f64::from(self.y + 1) / y_extent;
        let center = GeoPoint::from_degrees((north + south) * 0.5, (west + east) * 0.5);
        let radius = [
            GeoPoint::from_degrees(north, west),
            GeoPoint::from_degrees(north, east),
            GeoPoint::from_degrees(south, west),
            GeoPoint::from_degrees(south, east),
        ]
        .into_iter()
        .map(|corner| angular_distance(center.direction(), corner.direction()))
        .fold(0.0_f64, f64::max);
        (center, radius)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapTilePlacement {
    pub id: MapTileId,
    pub unwrapped_x: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TilePlan {
    pub level: u8,
    pub placements: Vec<MapTilePlacement>,
}

#[must_use]
pub fn plan_tiles(view: MapView, viewport_size: Vec2) -> TilePlan {
    if viewport_size.min_element() <= 1.0 {
        return TilePlan::default();
    }

    let pixels_per_vertical_world = viewport_size.y / view.vertical_span.max(f32::EPSILON);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let mut level = (pixels_per_vertical_world / MAP_TILE_CELLS_F32)
        .log2()
        .ceil()
        .clamp(0.0, f32::from(MAX_MAP_LEVEL)) as u8;

    loop {
        let placements = placements_at_level(view, level, viewport_size.x / viewport_size.y);
        if placements.len() <= MAX_VISIBLE_TILES || level == 0 {
            return TilePlan { level, placements };
        }
        level -= 1;
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)] // Map extents are capped at 2^18 and remain exact in f64.
fn placements_at_level(view: MapView, level: u8, aspect: f32) -> Vec<MapTilePlacement> {
    let y_extent = 1_i64 << level;
    let x_extent = y_extent * 2;
    let horizontal_span = view.horizontal_span(aspect);
    let x_min = ((view.center.x - f64::from(horizontal_span * 0.5)) * x_extent as f64).floor()
        as i64
        - PREFETCH_MARGIN;
    let x_max = ((view.center.x + f64::from(horizontal_span * 0.5)) * x_extent as f64).floor()
        as i64
        + PREFETCH_MARGIN;
    let y_min = (((view.center.y - f64::from(view.vertical_span * 0.5)) * y_extent as f64).floor()
        as i64
        - PREFETCH_MARGIN)
        .clamp(0, y_extent - 1);
    let y_max = (((view.center.y + f64::from(view.vertical_span * 0.5)) * y_extent as f64).floor()
        as i64
        + PREFETCH_MARGIN)
        .clamp(0, y_extent - 1);

    let mut placements = Vec::new();
    let mut seen = HashSet::new();
    for y in y_min..=y_max {
        for unwrapped_x in x_min..=x_max {
            let canonical_x = unwrapped_x.rem_euclid(x_extent);
            let placement = MapTilePlacement {
                id: MapTileId {
                    level,
                    x: u32::try_from(canonical_x).expect("canonical map tile x fits u32"),
                    y: u32::try_from(y).expect("clamped map tile y fits u32"),
                },
                unwrapped_x,
            };
            if seen.insert(placement) {
                placements.push(placement);
            }
        }
    }
    placements
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::DVec2;

    #[test]
    fn global_view_uses_square_root_tiles() {
        let plan = plan_tiles(MapView::default(), Vec2::new(1600.0, 800.0));
        assert!(plan.placements.len() <= MAX_VISIBLE_TILES);
        assert!(
            plan.placements
                .iter()
                .all(|tile| tile.id.x < tile.id.x_extent())
        );
        assert!(
            plan.placements
                .iter()
                .all(|tile| tile.id.y < tile.id.y_extent())
        );
    }

    #[test]
    fn planning_wraps_longitude_without_losing_placement() {
        let view = MapView {
            center: DVec2::new(0.01, 0.5),
            vertical_span: 0.25,
        };
        let plan = plan_tiles(view, Vec2::new(1200.0, 600.0));
        assert!(plan.placements.iter().any(|tile| tile.unwrapped_x < 0));
        assert!(
            plan.placements
                .iter()
                .all(|tile| tile.id.x < tile.id.x_extent())
        );
    }

    #[test]
    fn planning_across_longitude_seam_keeps_unwrapped_placements_stable() {
        let before = plan_tiles(
            MapView {
                center: DVec2::new(0.99, 0.5),
                vertical_span: 0.25,
            },
            Vec2::new(1_200.0, 600.0),
        );
        let after = plan_tiles(
            MapView {
                center: DVec2::new(1.01, 0.5),
                vertical_span: 0.25,
            },
            Vec2::new(1_200.0, 600.0),
        );

        let retained = before
            .placements
            .iter()
            .filter(|placement| after.placements.contains(placement))
            .count();
        assert!(retained > before.placements.len() / 2);
        assert!(
            after
                .placements
                .iter()
                .any(|placement| placement.unwrapped_x >= i64::from(placement.id.x_extent()))
        );
        assert!(
            after
                .placements
                .iter()
                .all(|placement| placement.id.x < placement.id.x_extent())
        );
    }

    #[test]
    fn zoom_is_capped_at_sub_metre_source_resolution() {
        let view = MapView {
            center: DVec2::splat(0.5),
            vertical_span: 1.0 / 262_144.0,
        };
        assert_eq!(
            plan_tiles(view, Vec2::new(1600.0, 800.0)).level,
            MAX_MAP_LEVEL
        );
    }
}
