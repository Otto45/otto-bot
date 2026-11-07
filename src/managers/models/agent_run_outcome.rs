use crate::managers::models::run_summary::RunSummary;
use crate::managers::models::work_plan::WorkPlan;

/// How the orchestrator's run ended: done, or with a plan the harness accepted, which the workflow carries on from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunOutcome {
    pub run_summary: RunSummary,
    pub work_plan: Option<WorkPlan>,
}
