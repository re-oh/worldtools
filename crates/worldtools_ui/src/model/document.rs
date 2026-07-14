use bevy::prelude::Resource;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SaveState {
    #[default]
    Saved,
    Modified,
    Saving,
    Failed,
}

#[derive(Resource, Debug, Clone)]
pub struct DocumentStatus {
    pub name: String,
    pub seed: u64,
    pub save_state: SaveState,
    pub can_undo: bool,
    pub can_redo: bool,
    pub path: Option<String>,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self {
            name: "Untitled World".to_owned(),
            seed: 1,
            save_state: SaveState::Saved,
            can_undo: false,
            can_redo: false,
            path: None,
        }
    }
}
