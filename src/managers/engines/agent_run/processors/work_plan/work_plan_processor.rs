use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::work_plan::WorkPlan;

// Checks the orchestrator's plan without a model, so the implementor starts from one that says what the change is and
// how to build it. Every problem is reported at once, so the orchestrator fixes them in one go.
pub struct WorkPlanProcessor;

impl WorkPlanProcessor {
    /// A rejected plan is an InvalidWorkPlan, whose message the orchestrator is told.
    pub fn check(&self, work_plan: &WorkPlan) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        if work_plan.summary.trim().is_empty() {
            errors.push("summary is empty".to_string());
        }
        if work_plan.steps.iter().all(|step: &String| step.trim().is_empty()) {
            errors.push("the plan has no steps: say how the implementor builds the change".to_string());
        }

        if errors.is_empty() {
            return Ok(());
        }
        Err(ApplicationBaseException::new(
            AgentRunEngineException::InvalidWorkPlan,
            format!(
                "The plan was not accepted:\n- {}\nFix all of these and submit the plan again.",
                errors.join("\n- ")
            ),
            ContextualData::new().with("error_count", i64::try_from(errors.len()).unwrap_or(i64::MAX)),
        ))
    }
}
