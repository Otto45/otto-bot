use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::implementation_brief::ImplementationBrief;
use crate::managers::models::review::Review;

/// What ends an agent's lane before its model stops on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneEnding {
    /// The orchestrator's plan, which the harness accepted.
    BriefAccepted(ImplementationBrief),
    /// The implementor's report.
    Reported(AgentReport),
    /// The reviewer's review.
    Reviewed(Review),
}
