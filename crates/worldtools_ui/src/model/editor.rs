use bevy::prelude::Resource;

use super::{ActiveTool, MapViewMode, WorldLayer};

#[derive(Resource, Debug, Clone, Default)]
pub struct EditorUiState {
    pub active_tool: ActiveTool,
    pub map_view: MapViewMode,
    pub active_layer: WorldLayer,
    pub analysis_open: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_starts_in_the_natural_terrain_view() {
        assert_eq!(EditorUiState::default().map_view, MapViewMode::Terrain);
    }

    #[test]
    fn analysis_drawer_starts_closed() {
        assert!(!EditorUiState::default().analysis_open);
    }
}
