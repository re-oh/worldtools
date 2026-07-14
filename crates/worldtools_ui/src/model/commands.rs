use bevy::prelude::Message;

use super::{
    ActiveTool, BrushSettings, GenerationScope, JobId, MapViewMode, ViewportRect, WorldLayer,
};

#[derive(Message, Debug, Clone, PartialEq)]
pub enum EditorCommand {
    NewWorld,
    OpenWorld,
    SaveWorld,
    SaveWorldAs,
    Undo,
    Redo,
    OpenPreferences,
    SelectTool(ActiveTool),
    SelectMapView(MapViewMode),
    SelectLayer(WorldLayer),
    SetLayerVisibility { layer: WorldLayer, visible: bool },
    SetLayerOpacity { layer: WorldLayer, opacity: f32 },
    UpdateBrush(BrushSettings),
    SetAutoRebuild(bool),
    Generate(GenerationScope),
    CancelJob(JobId),
    FocusDirtyRegion,
    ClearLayerEdits(WorldLayer),
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct MapViewportChanged {
    pub logical: ViewportRect,
    pub physical: ViewportRect,
    pub pixels_per_point: f32,
}
