use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ReviewFindingArguments {
    pub severity: Option<String>,
    pub path: Option<String>,
    pub problem: Option<String>,
    pub fix: Option<String>,
}
