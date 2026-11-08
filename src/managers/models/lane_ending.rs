use crate::managers::models::agent_report::AgentReport;
use crate::managers::models::review::Review;
use crate::managers::models::work_plan::WorkPlan;

/// What ends an agent's lane before its model stops on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneEnding {
    /// The orchestrator's plan, which the harness accepted.
    PlanAccepted(WorkPlan),
    /// The implementor's report.
    Reported(AgentReport),
    /// The reviewer's review.
    Reviewed(Review),
}
