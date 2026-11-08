use crate::managers::models::lane_ending::LaneEnding;
use crate::managers::models::run_summary::RunSummary;

/// How an agent's run ended: its model stopped on its own, or the lane was ended by an accepted plan or a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunOutcome {
    pub run_summary: RunSummary,
    pub lane_ending: Option<LaneEnding>,
}
