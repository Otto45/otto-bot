use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::implementation_brief::ImplementationBrief;

// Checks the orchestrator's brief without a model, so the implementor starts from one that says what to build and
// how it will know it is done. Every problem is reported at once, so the orchestrator fixes them in one go.
pub struct BriefProcessor;

impl BriefProcessor {
    /// A rejected brief is an InvalidBrief, whose message the orchestrator is told.
    pub fn check(&self, implementation_brief: &ImplementationBrief) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();
        if implementation_brief.task.trim().is_empty() {
            errors.push("task is empty: say what the implementor builds".to_string());
        }
        if implementation_brief.done_when.trim().is_empty() {
            errors.push("done_when is empty: say how the implementor knows the change is finished".to_string());
        }
        if implementation_brief
            .files
            .iter()
            .any(|path: &String| path.trim().is_empty())
        {
            errors.push("files has an empty path".to_string());
        }

        if errors.is_empty() {
            return Ok(());
        }
        Err(ApplicationBaseException::new(
            AgentRunEngineException::InvalidBrief,
            format!(
                "The brief was not accepted:\n- {}\nFix all of these and brief the implementor again.",
                errors.join("\n- ")
            ),
            ContextualData::new().with("error_count", i64::try_from(errors.len()).unwrap_or(i64::MAX)),
        ))
    }
}
