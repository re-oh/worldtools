use bevy::prelude::Resource;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainProbe {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
    pub elevation_m: f32,
    pub slope_degrees: f64,
    pub is_water: bool,
}

#[derive(Debug, Default, Resource)]
pub struct MapProbe {
    pub selected: Option<TerrainProbe>,
}
