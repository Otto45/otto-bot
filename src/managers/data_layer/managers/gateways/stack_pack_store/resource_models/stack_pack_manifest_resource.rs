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
    pub sources: Option<Vec<String>>,
    pub query: Option<String>,
}

// One rule; which fields it needs depends on its kind ("imports", "query", "named_after_file").
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LintRuleResource {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub message: Option<String>,
    pub files: Option<String>,
    pub forbid: Option<Vec<String>>,
    pub query: Option<String>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LintResource {
    pub sources: Option<Vec<String>>,
    pub imports_query: Option<String>,
    pub rules: Option<Vec<LintRuleResource>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct FormatResource {
    pub commands: Option<Vec<Vec<String>>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct StackPackManifestResource {
    pub name: Option<String>,
    pub language: Option<String>,
    pub rules: Option<String>,
    pub detect: Option<DetectResource>,
    pub checks: Option<Vec<CheckResource>>,
    pub format: Option<FormatResource>,
    pub project_map: Option<ProjectMapResource>,
    pub lint: Option<LintResource>,
}
