use bevy::prelude::Resource;

use super::{ActiveTool, BrushSettings, GenerationScope, WorldLayer};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    #[default]
    Jobs,
    Analysis,
}

#[derive(Resource, Debug, Clone)]
pub struct EditorUiState {
    pub active_tool: ActiveTool,
    pub active_layer: WorldLayer,
    pub layer_visibility: [bool; WorldLayer::COUNT],
    pub layer_opacity: [f32; WorldLayer::COUNT],
    pub brush: BrushSettings,
    pub generation_scope: GenerationScope,
    pub auto_rebuild: bool,
    pub drawer_tab: DrawerTab,
    pub drawer_open: bool,
}

impl Default for EditorUiState {
    fn default() -> Self {
        Self {
            active_tool: ActiveTool::default(),
            active_layer: WorldLayer::default(),
            layer_visibility: [true; WorldLayer::COUNT],
            layer_opacity: [1.0; WorldLayer::COUNT],
            brush: BrushSettings::default(),
            generation_scope: GenerationScope::Dirty,
            auto_rebuild: false,
            drawer_tab: DrawerTab::default(),
            drawer_open: false,
        }
    }
}

impl EditorUiState {
    #[must_use]
    pub fn layer_visible(&self, layer: WorldLayer) -> bool {
        self.layer_visibility[layer.index()]
    }

    pub fn set_layer_visible(&mut self, layer: WorldLayer, visible: bool) {
        self.layer_visibility[layer.index()] = visible;
    }

    #[must_use]
    pub fn layer_opacity(&self, layer: WorldLayer) -> f32 {
        self.layer_opacity[layer.index()]
    }

    pub fn set_layer_opacity(&mut self, layer: WorldLayer, opacity: f32) {
        self.layer_opacity[layer.index()] = opacity.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_state_is_indexed_by_layer() {
        let mut state = EditorUiState::default();
        state.set_layer_visible(WorldLayer::Climate, false);
        state.set_layer_opacity(WorldLayer::Hydrology, 0.4);

        assert!(!state.layer_visible(WorldLayer::Climate));
        assert!(state.layer_visible(WorldLayer::Elevation));
        assert!((state.layer_opacity(WorldLayer::Hydrology) - 0.4).abs() < f32::EPSILON);
    }
}
