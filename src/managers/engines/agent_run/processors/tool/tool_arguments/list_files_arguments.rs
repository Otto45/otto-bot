use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ListFilesArguments {
    pub directory: Option<String>,
}
