use bevy::prelude::Resource;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisSeverity {
    #[default]
    Information,
    Warning,
    Error,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AnalysisIssue {
    pub severity: AnalysisSeverity,
    pub label: String,
    pub location: Option<String>,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct AnalysisStatus {
    pub report_name: Option<String>,
    pub issues: Vec<AnalysisIssue>,
}
