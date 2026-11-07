use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WorkPackageArguments {
    pub name: Option<String>,
    pub brief: Option<String>,
    pub owned_paths: Option<Vec<String>>,
}
