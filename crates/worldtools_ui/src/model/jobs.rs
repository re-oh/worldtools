use bevy::prelude::Resource;

pub type JobId = u64;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum JobState {
    #[default]
    Queued,
    Running {
        progress: f32,
    },
    Complete,
    Failed {
        message: String,
    },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct JobSummary {
    pub id: JobId,
    pub label: String,
    pub state: JobState,
}

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct JobQueue {
    pub jobs: Vec<JobSummary>,
}

impl JobQueue {
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| matches!(job.state, JobState::Queued | JobState::Running { .. }))
            .count()
    }
}
