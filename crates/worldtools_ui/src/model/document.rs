use bevy::prelude::Resource;

#[derive(Resource, Debug, Clone)]
pub struct DocumentStatus {
    pub name: String,
    pub seed: u64,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self {
            name: "Untitled World".to_owned(),
            seed: 1,
        }
    }
}
