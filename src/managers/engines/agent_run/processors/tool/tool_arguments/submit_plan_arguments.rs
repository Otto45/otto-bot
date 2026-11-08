use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SubmitPlanArguments {
    pub summary: Option<String>,
    pub assumptions: Option<Vec<String>>,
    pub steps: Option<Vec<String>>,
}
