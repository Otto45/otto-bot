use crate::managers::models::lint_rule::LintRule;

/// How a stack pack checks a project against the architecture: which files (globs), the query that captures each
/// file's imports (@import), and the rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackPackLint {
    pub source_globs: Vec<String>,
    pub imports_query: String,
    pub rules: Vec<LintRule>,
}
