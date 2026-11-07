use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlannedComponentArguments {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub within: Option<String>,
}
