use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScaffoldArguments {
    pub kind: Option<String>,
    pub name: Option<String>,
    pub within: Option<String>,
}
