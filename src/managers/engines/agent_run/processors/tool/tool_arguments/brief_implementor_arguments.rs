use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BriefImplementorArguments {
    pub task: Option<String>,
    pub decisions: Option<Vec<String>>,
    pub done_when: Option<String>,
    pub files: Option<Vec<String>>,
}
