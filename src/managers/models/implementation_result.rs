use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::run_summary::RunSummary;

/// How the implementor's build of a plan went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationResult {
    /// None when the implementor finished without reporting.
    pub report: Option<AgentReport>,
    /// Whether every check passed when it finished.
    pub checks_passed: bool,
    pub run_summary: RunSummary,
}
