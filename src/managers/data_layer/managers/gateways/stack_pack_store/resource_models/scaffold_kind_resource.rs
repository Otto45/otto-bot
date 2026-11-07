use serde::Deserialize;

// One scaffold kind's file (scaffolds/<kind>.toml), deserialized as it is, every field optional.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScaffoldFileResource {
    pub path: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScaffoldInsertionResource {
    pub path: Option<String>,
    pub before: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ScaffoldKindResource {
    pub description: Option<String>,
    pub within: Option<String>,
    pub files: Option<Vec<ScaffoldFileResource>>,
    pub insertions: Option<Vec<ScaffoldInsertionResource>>,
}
