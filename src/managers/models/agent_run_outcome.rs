use crate::managers::models::lane_ending::LaneEnding;
use crate::managers::models::run_summary::RunSummary;

/// How an agent's run ended: its model stopped on its own, or the lane was ended by an accepted plan or a report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunOutcome {
    pub run_summary: RunSummary,
    pub lane_ending: Option<LaneEnding>,
    /// How many input tokens the lane's last request had: how long its conversation has grown.
    pub last_input_tokens: i64,
}
