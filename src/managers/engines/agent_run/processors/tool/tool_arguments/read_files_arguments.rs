use serde::Deserialize;

// A function tool's arguments as the model sent them, every field optional so what is wrong can be said.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ReadFilesArguments {
    pub paths: Option<Vec<String>>,
}
