use std::collections::HashMap;
use std::path::Path;

use crate::managers::data_layer::managers::gateways::stack_pack_store::resource_models::stack_pack_manifest_resource::{
    CheckResource, DetectResource, FormatResource, LintResource, LintRuleResource, ProjectMapResource,
    ScaffoldResource, StackPackManifestResource,
};
use crate::managers::models::lint_rule::LintRule;
use crate::managers::models::stack_pack::StackPack;
use crate::managers::models::stack_pack_check::StackPackCheck;
use crate::managers::models::stack_pack_format::StackPackFormat;
use crate::managers::models::stack_pack_lint::StackPackLint;
use crate::managers::models::scaffold_kind::ScaffoldKind;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::stack_pack_scaffold::StackPackScaffold;

pub struct StackPackMapper;

impl StackPackMapper {
    /// Expects a manifest that has already passed ValidatorStackPackManifestResource: each `expect` states a checked
    /// fact, and fails only if the validator is wrong. `queries` holds the text of every query file the manifest
    /// names, by the path it names it with, read by the gateway; `scaffold_kinds` the pack's scaffold kinds, read and
    /// mapped by the gateway.
    pub fn to_stack_pack(
        manifest: StackPackManifestResource,
        pack_directory: &Path,
        queries: &HashMap<String, String>,
        scaffold_kinds: Vec<ScaffoldKind>,
    ) -> StackPack {
        let detect: DetectResource = manifest
            .detect
            .expect("validated by ValidatorStackPackManifestResource");
        let get_query = |query_path: Option<String>| -> String {
            let query_path: String = query_path.expect("validated by ValidatorStackPackManifestResource");
            queries
                .get(&query_path)
                .cloned()
                .expect("read by StackPackStoreGateway for every query path")
        };

        StackPack {
            name: manifest.name.expect("validated by ValidatorStackPackManifestResource"),
            language: manifest.language,
            detect_files: detect.files.expect("validated by ValidatorStackPackManifestResource"),
            rules_path: pack_directory.join(manifest.rules.expect("validated by ValidatorStackPackManifestResource")),
            checks: manifest
                .checks
                .unwrap_or_default()
                .into_iter()
                .map(|check: CheckResource| StackPackCheck {
                    name: check.name.expect("validated by ValidatorStackPackManifestResource"),
                    command: check.command.expect("validated by ValidatorStackPackManifestResource"),
                })
                .collect(),
            project_map: manifest
                .project_map
                .map(|project_map: ProjectMapResource| StackPackProjectMap {
                    source_globs: project_map
                        .sources
                        .expect("validated by ValidatorStackPackManifestResource"),
                    query: get_query(project_map.query),
                }),
            lint: manifest.lint.map(|lint: LintResource| StackPackLint {
                source_globs: lint.sources.expect("validated by ValidatorStackPackManifestResource"),
                imports_query: get_query(lint.imports_query),
                rules: lint
                    .rules
                    .unwrap_or_default()
                    .into_iter()
                    .map(|lint_rule: LintRuleResource| StackPackMapper::to_lint_rule(lint_rule, &get_query))
                    .collect(),
            }),
            scaffold: manifest
                .scaffold
                .map(|_: ScaffoldResource| StackPackScaffold { kinds: scaffold_kinds }),
            format: manifest.format.map(|format: FormatResource| StackPackFormat {
                commands: format
                    .commands
                    .expect("validated by ValidatorStackPackManifestResource"),
                source_globs: format.sources.expect("validated by ValidatorStackPackManifestResource"),
            }),
        }
    }

    fn to_lint_rule(lint_rule: LintRuleResource, get_query: &dyn Fn(Option<String>) -> String) -> LintRule {
        let name: String = lint_rule.name.expect("validated by ValidatorStackPackManifestResource");
        let message: String = lint_rule
            .message
            .expect("validated by ValidatorStackPackManifestResource");
        match lint_rule
            .kind
            .as_deref()
            .expect("validated by ValidatorStackPackManifestResource")
        {
            "imports" => LintRule::Imports {
                name,
                message,
                file_pattern: lint_rule
                    .files
                    .expect("validated by ValidatorStackPackManifestResource"),
                forbidden_import_patterns: lint_rule
                    .forbid
                    .expect("validated by ValidatorStackPackManifestResource"),
            },
            "query" => LintRule::Query {
                name,
                message,
                query: get_query(lint_rule.query),
                exclude_globs: lint_rule.exclude.unwrap_or_default(),
            },
            "named_after_file" => LintRule::NamedAfterFile {
                name,
                message,
                query: get_query(lint_rule.query),
                exclude_globs: lint_rule.exclude.unwrap_or_default(),
            },
            other => unreachable!("validated by ValidatorStackPackManifestResource: {other}"),
        }
    }
}
