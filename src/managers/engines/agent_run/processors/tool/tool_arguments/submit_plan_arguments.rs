use serde::Deserialize;

use crate::managers::engines::agent_run::processors::tool::tool_arguments::planned_component_arguments::PlannedComponentArguments;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SubmitPlanArguments {
    pub summary: Option<String>,
    pub assumptions: Option<Vec<String>>,
    pub components: Option<Vec<PlannedComponentArguments>>,
    pub steps: Option<Vec<String>>,
}
