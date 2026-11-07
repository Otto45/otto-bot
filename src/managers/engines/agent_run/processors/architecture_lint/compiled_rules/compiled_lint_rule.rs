use globset::GlobSet;
use regex::Regex;
use tree_sitter::Query;

use crate::managers::models::lint_rule::LintRule;

/// A rule ready to check files with: its patterns and queries compiled once for every file.
pub enum CompiledLintRule {
    Imports {
        lint_rule: LintRule,
        file_pattern: Regex,
        forbidden_import_patterns: Vec<Regex>,
    },
    Query {
        lint_rule: LintRule,
        query: Query,
        excluded: GlobSet,
    },
    NamedAfterFile {
        lint_rule: LintRule,
        query: Query,
        excluded: GlobSet,
    },
}
