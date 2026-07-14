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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GenerationScope {
    World,
    Visible,
    #[default]
    Dirty,
}

impl GenerationScope {
    pub const ALL: [Self; 3] = [Self::Dirty, Self::Visible, Self::World];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::World => "Entire world",
            Self::Visible => "Visible region",
            Self::Dirty => "Dirty tiles",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum GenerationActivity {
    #[default]
    Idle,
    Queued {
        jobs: u32,
    },
    Running {
        stage: PipelineStage,
        completed: u32,
        total: u32,
    },
    Failed {
        message: String,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    pub tile_count: u32,
    pub from_stage: Option<PipelineStage>,
}

impl DirtyRegion {
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.tile_count == 0
    }
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct GenerationStatus {
    pub activity: GenerationActivity,
    pub dirty: DirtyRegion,
}
