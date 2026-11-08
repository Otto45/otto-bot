use std::collections::HashSet;

use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::planned_component::PlannedComponent;
use crate::managers::models::work_plan::WorkPlan;

// Checks the orchestrator's plan without a model, so the implementor starts from one that holds up: it says what the
// change is and how to build it, and its components can be scaffolded. Every problem is reported at once, so the
// orchestrator fixes them in one go.
pub struct WorkPlanProcessor;

impl WorkPlanProcessor {
    /// `component_errors` are what is wrong with the components themselves (a kind the stack doesn't have, a file that
    /// exists), found by scaffolding them on paper. A rejected plan is an InvalidWorkPlan, whose message the
    /// orchestrator is told.
    pub fn check(&self, work_plan: &WorkPlan, component_errors: Vec<String>) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        if work_plan.summary.trim().is_empty() {
            errors.push("summary is empty".to_string());
        }
        if work_plan.steps.iter().all(|step: &String| step.trim().is_empty()) {
            errors.push("the plan has no steps: say how the implementor builds the change".to_string());
        }
        errors.extend(WorkPlanProcessor::check_components(&work_plan.components));
        errors.extend(component_errors);

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

    fn check_components(planned_components: &[PlannedComponent]) -> Vec<String> {
        let mut errors: Vec<String> = Vec::new();
        let mut seen: HashSet<(&str, &str, Option<&str>)> = HashSet::new();
        for planned_component in planned_components {
            let key: (&str, &str, Option<&str>) = (
                planned_component.kind.as_str(),
                planned_component.name.as_str(),
                planned_component.within.as_deref(),
            );
            if !seen.insert(key) {
                errors.push(format!(
                    "the {} {} is listed twice",
                    planned_component.kind, planned_component.name
                ));
            }
        }
        errors
    }
}
