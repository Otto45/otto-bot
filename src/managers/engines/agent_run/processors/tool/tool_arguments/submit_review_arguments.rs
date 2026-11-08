use serde::Deserialize;

use crate::managers::engines::agent_run::processors::tool::tool_arguments::review_finding_arguments::ReviewFindingArguments;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SubmitReviewArguments {
    pub summary: Option<String>,
    pub findings: Option<Vec<ReviewFindingArguments>>,
}
