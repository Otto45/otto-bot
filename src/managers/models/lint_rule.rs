/// One of a stack pack's architecture rules, checked without a model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintRule {
    /// A file whose path matches `file_pattern` (a regular expression) imports nothing matching any of
    /// `forbidden_import_patterns`. When `file_pattern` and a forbidden pattern both have a group named `name`, an
    /// import breaks the rule only when the two names differ (e.g. an engine importing another engine).
    Imports {
        name: String,
        message: String,
        file_pattern: String,
        forbidden_import_patterns: Vec<String>,
    },
    /// What `query` captures as @violation, except the nodes it also captures as @allowed, breaks the rule.
    Query {
        name: String,
        message: String,
        query: String,
        exclude_globs: Vec<String>,
    },
    /// A file holds a type named after it: one of the names `query` captures as @name is the file's name in
    /// PascalCase (`greeting_processor.rs` holds `GreetingProcessor`).
    NamedAfterFile {
        name: String,
        message: String,
        query: String,
        exclude_globs: Vec<String>,
    },
}

impl LintRule {
    pub fn name(&self) -> &str {
        match self {
            LintRule::Imports { name, .. } | LintRule::Query { name, .. } | LintRule::NamedAfterFile { name, .. } => {
                name
            }
        }
    }
}
