use globset::{Glob, GlobSet, GlobSetBuilder};
use tree_sitter::{Language, Query};

use crate::managers::engines::agent_run::processors::project_map::signature_extractors::tree_sitter_signature_extractor::TreeSitterSignatureExtractor;
use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::workspace_file::WorkspaceFile;

// Writes the project map: every file of the workspace, and under each source file its public items with their bodies
// left out. In a Domain Container app a file holds the one type it is named after, so a file's items need no other
// label. The map is rendered the same way every time (sorted paths, items in source order), so it caches.
pub struct ProjectMapProcessor;

impl ProjectMapProcessor {
    /// The paths the pack's source globs match, in the order given.
    pub fn select_source_paths(
        &self,
        paths: &[String],
        stack_pack_project_map: &StackPackProjectMap,
    ) -> ApplicationResult<Vec<String>> {
        let mut glob_set_builder: GlobSetBuilder = GlobSetBuilder::new();
        for source_glob in &stack_pack_project_map.source_globs {
            glob_set_builder.add(Glob::new(source_glob).map_err(|exception: globset::Error| {
                ProjectMapProcessor::to_invalid_project_map(format!("project_map.sources: {exception}"))
            })?);
        }
        let glob_set: GlobSet = glob_set_builder.build().map_err(|exception: globset::Error| {
            ProjectMapProcessor::to_invalid_project_map(format!("project_map.sources: {exception}"))
        })?;

        Ok(paths
            .iter()
            .filter(|path: &&String| glob_set.is_match(path.as_str()))
            .cloned()
            .collect())
    }

    /// `paths` is every file of the workspace; `source_files` the ones the pack's globs select, read.
    pub fn build_project_map(
        &self,
        paths: &[String],
        source_files: &[WorkspaceFile],
        stack_pack_project_map: &StackPackProjectMap,
    ) -> ApplicationResult<String> {
        let language: Language = TreeSitterSignatureExtractor::get_language(&stack_pack_project_map.language)?;
        let query: Query = TreeSitterSignatureExtractor::compile_query(&language, &stack_pack_project_map.query)?;

        let mut sorted_paths: Vec<&String> = paths.iter().collect();
        sorted_paths.sort();

        let mut lines: Vec<String> = Vec::with_capacity(paths.len());
        for path in sorted_paths {
            lines.push(path.clone());
            let source_file: Option<&WorkspaceFile> = source_files
                .iter()
                .find(|source_file: &&WorkspaceFile| &source_file.path == path);
            if let Some(source_file) = source_file {
                for item in TreeSitterSignatureExtractor::extract(&language, &query, &source_file.content)? {
                    lines.push(format!("    {item}"));
                }
            }
        }
        Ok(lines.join("\n"))
    }

    fn to_invalid_project_map(errors: String) -> ApplicationBaseException {
        ApplicationBaseException::new(
            AgentRunEngineException::InvalidProjectMap,
            "The stack pack's project map is invalid",
            ContextualData::new().with("errors", errors),
        )
    }
}
