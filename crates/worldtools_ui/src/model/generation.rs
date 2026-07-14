use std::fmt;

use bevy::prelude::Resource;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    #[default]
    BaseShape,
    Tectonics,
    Erosion,
    Hydrology,
    Climate,
    Surface,
    Resources,
}

impl fmt::Display for PipelineStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BaseShape => "base shape",
            Self::Tectonics => "tectonics",
            Self::Erosion => "erosion",
            Self::Hydrology => "hydrology",
            Self::Climate => "climate",
            Self::Surface => "surface",
            Self::Resources => "resources",
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum GenerationActivity {
    #[default]
    Idle,
    Running {
        stage: PipelineStage,
        completed: u32,
        total: u32,
    },
    Failed {
        message: String,
    },
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct GenerationStatus {
    pub activity: GenerationActivity,
}

impl GenerationStatus {
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(&self.activity, GenerationActivity::Running { .. })
    }
}

/// Editable generation parameters which are not active until regeneration
/// succeeds. This prevents the displayed seed from disagreeing with the
/// immutable snapshot currently being inspected.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldGenerationDraft {
    pub seed: u64,
}

impl Default for WorldGenerationDraft {
    fn default() -> Self {
        Self { seed: 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_draft_matches_the_initial_document_seed() {
        assert_eq!(WorldGenerationDraft::default().seed, 1);
    }

    #[test]
    fn only_active_world_generation_disables_regeneration() {
        assert!(!GenerationStatus::default().is_running());
        assert!(
            GenerationStatus {
                activity: GenerationActivity::Running {
                    stage: PipelineStage::Tectonics,
                    completed: 0,
                    total: 1,
                },
            }
            .is_running()
        );
        assert!(
            !GenerationStatus {
                activity: GenerationActivity::Failed {
                    message: "failed".to_owned(),
                },
            }
            .is_running()
        );
    }
}
