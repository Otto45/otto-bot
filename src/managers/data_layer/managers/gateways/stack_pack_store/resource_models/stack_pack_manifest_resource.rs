use serde::Deserialize;

// A pack's pack.toml, deserialized as it is, every field optional so the validator can say what is wrong with it.
// Fields the harness doesn't use yet (e.g. the rename map) are left out.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DetectResource {
    pub files: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CheckResource {
    pub name: Option<String>,
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ProjectMapResource {
    pub language: Option<String>,
    pub sources: Option<Vec<String>>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StackPackManifestResource {
    pub name: Option<String>,
    pub rules: Option<String>,
    pub detect: Option<DetectResource>,
    pub checks: Option<Vec<CheckResource>>,
    pub project_map: Option<ProjectMapResource>,
}
