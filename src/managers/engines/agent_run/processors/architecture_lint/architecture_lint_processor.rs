use std::path::Path;

use regex::{Captures, Regex};
use tree_sitter::{Language, Query, Tree};

use crate::managers::common::matchers::path_glob_matcher::PathGlobMatcher;
use crate::managers::common::tree_sitter::captured_node::CapturedNode;
use crate::managers::common::tree_sitter::tree_sitter_query_runner::TreeSitterQueryRunner;
use crate::managers::engines::agent_run::processors::architecture_lint::compiled_rules::compiled_lint_rule::CompiledLintRule;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::models::lint_rule::LintRule;
use crate::managers::models::lint_violation::LintViolation;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::workspace_file::WorkspaceFile;

const IMPORT_CAPTURE: &str = "import";
const VIOLATION_CAPTURE: &str = "violation";
const ALLOWED_CAPTURE: &str = "allowed";
const NAME_CAPTURE: &str = "name";
// The group an import rule's patterns share: "another one of the same kind".
const SAME_NAME_GROUP: &str = "name";
const MAX_EXCERPT_CHARS: usize = 100;

// Checks source files against the stack pack's architecture rules, deterministically: no model, so a rule broken is
// found for nothing and told to the agent with the checks.
pub struct ArchitectureLintProcessor;

impl ArchitectureLintProcessor {
    /// Every violation in the files, by path and line.
    pub fn lint(
        &self,
        source_files: &[WorkspaceFile],
        language_name: &str,
        stack_pack_lint: &StackPackLint,
    ) -> ApplicationResult<Vec<LintViolation>> {
        let language: Language = TreeSitterQueryRunner::get_language(language_name)?;
        let imports_query: Query =
            TreeSitterQueryRunner::compile_query(&language, &stack_pack_lint.imports_query, "lint imports")?;
        let compiled_rules: Vec<CompiledLintRule> = stack_pack_lint
            .rules
            .iter()
            .map(|lint_rule: &LintRule| ArchitectureLintProcessor::compile_rule(&language, lint_rule))
            .collect::<ApplicationResult<Vec<CompiledLintRule>>>()?;

        let mut lint_violations: Vec<LintViolation> = Vec::new();
        for source_file in source_files {
            let Some(tree) = TreeSitterQueryRunner::parse(&language, &source_file.content)? else {
                continue;
            };
            let tree: &Tree = &tree;
            let imports: Vec<CapturedNode> = TreeSitterQueryRunner::capture(&imports_query, tree, &source_file.content)
                .into_iter()
                .filter(|captured_node: &CapturedNode| captured_node.capture_name == IMPORT_CAPTURE)
                .collect();
            for compiled_rule in &compiled_rules {
                lint_violations.extend(ArchitectureLintProcessor::check_rule(
                    compiled_rule,
                    source_file,
                    tree,
                    &imports,
                ));
            }
        }
        lint_violations.sort_by(|left: &LintViolation, right: &LintViolation| {
            (&left.path, left.line_number, &left.rule_name).cmp(&(&right.path, right.line_number, &right.rule_name))
        });
        Ok(lint_violations)
    }

    fn compile_rule(language: &Language, lint_rule: &LintRule) -> ApplicationResult<CompiledLintRule> {
        Ok(match lint_rule {
            LintRule::Imports {
                file_pattern,
                forbidden_import_patterns,
                ..
            } => CompiledLintRule::Imports {
                lint_rule: lint_rule.clone(),
                file_pattern: ArchitectureLintProcessor::compile_pattern(lint_rule, file_pattern)?,
                forbidden_import_patterns: forbidden_import_patterns
                    .iter()
                    .map(|pattern: &String| ArchitectureLintProcessor::compile_pattern(lint_rule, pattern))
                    .collect::<ApplicationResult<Vec<Regex>>>()?,
            },
            LintRule::Query {
                query, exclude_globs, ..
            } => CompiledLintRule::Query {
                lint_rule: lint_rule.clone(),
                query: TreeSitterQueryRunner::compile_query(language, query, lint_rule.name())?,
                excluded: PathGlobMatcher::build(exclude_globs)?,
            },
            LintRule::NamedAfterFile {
                query, exclude_globs, ..
            } => CompiledLintRule::NamedAfterFile {
                lint_rule: lint_rule.clone(),
                query: TreeSitterQueryRunner::compile_query(language, query, lint_rule.name())?,
                excluded: PathGlobMatcher::build(exclude_globs)?,
            },
        })
    }

    fn check_rule(
        compiled_rule: &CompiledLintRule,
        source_file: &WorkspaceFile,
        tree: &Tree,
        imports: &[CapturedNode],
    ) -> Vec<LintViolation> {
        let source: &str = &source_file.content;
        match compiled_rule {
            CompiledLintRule::Imports {
                lint_rule,
                file_pattern,
                forbidden_import_patterns,
            } => {
                let Some(file_captures) = file_pattern.captures(&source_file.path) else {
                    return Vec::new();
                };
                let file_name: Option<&str> = file_captures
                    .name(SAME_NAME_GROUP)
                    .map(|name: regex::Match<'_>| name.as_str());
                imports
                    .iter()
                    .filter(|import: &&CapturedNode| {
                        let import_text: String =
                            ArchitectureLintProcessor::to_one_line(&source[import.byte_range.clone()]);
                        forbidden_import_patterns
                            .iter()
                            .any(|forbidden_import_pattern: &Regex| {
                                ArchitectureLintProcessor::is_forbidden(
                                    forbidden_import_pattern,
                                    &import_text,
                                    file_name,
                                )
                            })
                    })
                    .map(|import: &CapturedNode| {
                        ArchitectureLintProcessor::to_violation(lint_rule, source_file, import)
                    })
                    .collect()
            }
            CompiledLintRule::Query {
                lint_rule,
                query,
                excluded,
            } => {
                if excluded.is_match(&source_file.path) {
                    return Vec::new();
                }
                let captured_nodes: Vec<CapturedNode> = TreeSitterQueryRunner::capture(query, tree, source);
                let allowed: Vec<&CapturedNode> = captured_nodes
                    .iter()
                    .filter(|captured_node: &&CapturedNode| captured_node.capture_name == ALLOWED_CAPTURE)
                    .collect();
                let mut violations: Vec<&CapturedNode> = captured_nodes
                    .iter()
                    .filter(|captured_node: &&CapturedNode| {
                        captured_node.capture_name == VIOLATION_CAPTURE
                            && !allowed
                                .iter()
                                .any(|allowed_node: &&CapturedNode| allowed_node.byte_range == captured_node.byte_range)
                    })
                    .collect();
                violations.dedup_by_key(|captured_node: &mut &CapturedNode| captured_node.byte_range.clone());
                violations
                    .into_iter()
                    .map(|captured_node: &CapturedNode| {
                        ArchitectureLintProcessor::to_violation(lint_rule, source_file, captured_node)
                    })
                    .collect()
            }
            CompiledLintRule::NamedAfterFile {
                lint_rule,
                query,
                excluded,
            } => {
                if excluded.is_match(&source_file.path) {
                    return Vec::new();
                }
                let expected_name: String = ArchitectureLintProcessor::to_type_name(&source_file.path);
                let type_names: Vec<String> = TreeSitterQueryRunner::capture(query, tree, source)
                    .iter()
                    .filter(|captured_node: &&CapturedNode| captured_node.capture_name == NAME_CAPTURE)
                    .map(|captured_node: &CapturedNode| source[captured_node.byte_range.clone()].to_string())
                    .collect();
                // A file of functions only (an entry point) declares no type to be named after. Case is ignored, so
                // `openai_responses.rs` may hold `OpenAiResponses`.
                if type_names.is_empty()
                    || type_names
                        .iter()
                        .any(|type_name: &String| type_name.eq_ignore_ascii_case(&expected_name))
                {
                    return Vec::new();
                }
                vec![LintViolation {
                    rule_name: lint_rule.name().to_string(),
                    message: format!(
                        "{} (expected {expected_name})",
                        ArchitectureLintProcessor::get_message(lint_rule)
                    ),
                    path: source_file.path.clone(),
                    line_number: 1,
                    excerpt: String::new(),
                }]
            }
        }
    }

    // A forbidden pattern that shares the file pattern's `name` group forbids only another of the same kind: the
    // import breaks the rule when the names differ.
    fn is_forbidden(forbidden_import_pattern: &Regex, import_text: &str, file_name: Option<&str>) -> bool {
        forbidden_import_pattern
            .captures_iter(import_text)
            .any(|import_captures: Captures<'_>| {
                match (
                    file_name,
                    import_captures
                        .name(SAME_NAME_GROUP)
                        .map(|name: regex::Match<'_>| name.as_str()),
                ) {
                    (Some(file_name), Some(import_name)) => file_name != import_name,
                    _ => true,
                }
            })
    }

    fn to_violation(lint_rule: &LintRule, source_file: &WorkspaceFile, captured_node: &CapturedNode) -> LintViolation {
        let text: String =
            ArchitectureLintProcessor::to_one_line(&source_file.content[captured_node.byte_range.clone()]);
        LintViolation {
            rule_name: lint_rule.name().to_string(),
            message: ArchitectureLintProcessor::get_message(lint_rule).to_string(),
            path: source_file.path.clone(),
            line_number: captured_node.line_number,
            excerpt: text.chars().take(MAX_EXCERPT_CHARS).collect(),
        }
    }

    fn get_message(lint_rule: &LintRule) -> &str {
        match lint_rule {
            LintRule::Imports { message, .. }
            | LintRule::Query { message, .. }
            | LintRule::NamedAfterFile { message, .. } => message,
        }
    }

    // `src/managers/processors/greeting/greeting_processor.rs` -> `GreetingProcessor`.
    fn to_type_name(path: &str) -> String {
        let file_stem: String = Path::new(path)
            .file_stem()
            .map(|file_stem: &std::ffi::OsStr| file_stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        file_stem
            .split(['_', '-', '.'])
            .filter(|word: &&str| !word.is_empty())
            .map(|word: &str| {
                let mut characters: std::str::Chars<'_> = word.chars();
                match characters.next() {
                    Some(first) => first.to_uppercase().chain(characters).collect::<String>(),
                    None => String::new(),
                }
            })
            .collect()
    }

    fn compile_pattern(lint_rule: &LintRule, pattern: &str) -> ApplicationResult<Regex> {
        Regex::new(pattern).map_err(|exception: regex::Error| {
            ApplicationBaseException::new(
                ExceptionKind::InvalidStackPack,
                "A pattern of the stack pack's lint rules is not a valid regular expression",
                ContextualData::new()
                    .with("rule_name", lint_rule.name())
                    .with("pattern", pattern)
                    .with("errors", exception.to_string()),
            )
        })
    }

    fn to_one_line(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }
}
