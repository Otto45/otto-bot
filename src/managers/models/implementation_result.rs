use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::review::Review;
use crate::managers::models::run_summary::RunSummary;

/// How the build of a plan went: the implementor's last report, whether every check passed at the end, and the
/// reviewer's review of the change (none when the build didn't get that far).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplementationResult {
    /// None when the implementor finished without reporting.
    pub report: Option<AgentReport>,
    pub checks_passed: bool,
    pub review: Option<Review>,
    /// Whether the implementor was sent the review's high-severity findings to fix.
    pub fixed_after_review: bool,
    pub run_summary: RunSummary,
}
