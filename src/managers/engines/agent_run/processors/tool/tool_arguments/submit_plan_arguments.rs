use serde::Deserialize;

use crate::managers::engines::agent_run::processors::tool::tool_arguments::planned_component_arguments::PlannedComponentArguments;
use crate::managers::engines::agent_run::processors::tool::tool_arguments::work_package_arguments::WorkPackageArguments;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SubmitPlanArguments {
    pub summary: Option<String>,
    pub assumptions: Option<Vec<String>>,
    pub components: Option<Vec<PlannedComponentArguments>>,
    pub packages: Option<Vec<WorkPackageArguments>>,
}
