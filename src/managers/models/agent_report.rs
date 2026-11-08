use crate::managers::models::report_status::ReportStatus;

/// What the implementor says when its build of a plan ends: terse, for the person and the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentReport {
    pub status: ReportStatus,
    pub summary: String,
    /// The changes it needs that it may not make itself, each saying where and why.
    pub needs: Vec<String>,
}
