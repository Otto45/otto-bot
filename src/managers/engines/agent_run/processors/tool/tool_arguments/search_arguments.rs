use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SearchArguments {
    pub pattern: Option<String>,
    pub path_glob: Option<String>,
}
