use std::ops::Range;

use tree_sitter::{Language, Query, Tree};

use crate::managers::common::tree_sitter::captured_node::CapturedNode;
use crate::managers::common::tree_sitter::tree_sitter_query_runner::TreeSitterQueryRunner;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::models::stack_pack_project_map::StackPackProjectMap;
use crate::managers::models::workspace_file::WorkspaceFile;

const ITEM_INDENT: &str = "    ";
const ITEM_CAPTURE: &str = "item";
// What an item leaves out: its body, and anything else the query marks (e.g. comments).
const LEFT_OUT_CAPTURES: [&str; 2] = ["body", "omit"];
// The map is rebuilt once the notes describing its changes would be more than a quarter of it.
const REBUILD_FRACTION_DENOMINATOR: usize = 4;

// Writes the project map: every file of the workspace, and under each source file its public items with their bodies
// left out, one per line, picked by the stack pack's query (its @item captures, without the @body and @omit captures
// inside them). In a Domain Container app a file holds the one type it is named after, so a file's items need no other
// label. The map is rendered the same way every time (sorted paths, items in source order), so it caches.
pub struct ProjectMapProcessor;

impl ProjectMapProcessor {
    /// `paths` is every file of the workspace; `source_files` the ones the pack's globs select, read.
    pub fn build_project_map(
        &self,
        paths: &[String],
        source_files: &[WorkspaceFile],
        language_name: &str,
        stack_pack_project_map: &StackPackProjectMap,
    ) -> ApplicationResult<String> {
        let language: Language = TreeSitterQueryRunner::get_language(language_name)?;
        let query: Query =
            TreeSitterQueryRunner::compile_query(&language, &stack_pack_project_map.query, "project map")?;
        if query.capture_index_for_name(ITEM_CAPTURE).is_none() {
            return Err(ApplicationBaseException::new(
                ExceptionKind::InvalidStackPack,
                "The stack pack's project map query captures no @item",
                ContextualData::new(),
            ));
        }

        let mut sorted_paths: Vec<&String> = paths.iter().collect();
        sorted_paths.sort();

        let mut lines: Vec<String> = Vec::with_capacity(paths.len());
        for path in sorted_paths {
            lines.push(path.clone());
            let source_file: Option<&WorkspaceFile> = source_files
                .iter()
                .find(|source_file: &&WorkspaceFile| &source_file.path == path);
            if let Some(source_file) = source_file {
                for item in ProjectMapProcessor::extract_items(&language, &query, &source_file.content)? {
                    lines.push(format!("{ITEM_INDENT}{item}"));
                }
            }
        }
        Ok(lines.join("\n"))
    }

    /// What changed in the map from `since` to `current`, as a note for the model: each changed or added file's entry
    /// as it is now, and each removed file. Empty when nothing changed.
    pub fn describe_changes(&self, since: &str, current: &str) -> String {
        let since_entries: Vec<(String, String)> = ProjectMapProcessor::to_entries(since);
        let current_entries: Vec<(String, String)> = ProjectMapProcessor::to_entries(current);
        let find = |entries: &[(String, String)], path: &str| -> Option<String> {
            entries
                .iter()
                .find(|(entry_path, _): &&(String, String)| entry_path == path)
                .map(|(_, entry): &(String, String)| entry.clone())
        };

        let mut changes: Vec<String> = Vec::new();
        for (path, entry) in &current_entries {
            match find(&since_entries, path) {
                None => changes.push(format!("added {entry}")),
                Some(since_entry) if &since_entry != entry => changes.push(format!("changed {entry}")),
                Some(_) => {}
            }
        }
        for (path, _) in &since_entries {
            if find(&current_entries, path).is_none() {
                changes.push(format!("removed {path}"));
            }
        }

        if changes.is_empty() {
            return String::new();
        }
        format!(
            "The workspace changed since the project map was built. These entries replace the map's own (each \
             file's entry as it is now):\n\n{}",
            changes.join("\n")
        )
    }

    /// Whether the map changed so much from the one the requests open with that it is rebuilt (breaking the cache
    /// from it on) rather than described in notes.
    pub fn should_rebuild(&self, layer_map: &str, current: &str) -> bool {
        self.describe_changes(layer_map, current).len() * REBUILD_FRACTION_DENOMINATOR > current.len()
    }

    // The map's entries: each file's path, and its entry (the path's line and its items' lines).
    fn to_entries(project_map: &str) -> Vec<(String, String)> {
        let mut entries: Vec<(String, String)> = Vec::new();
        for line in project_map.lines() {
            match (line.starts_with(ITEM_INDENT), entries.last_mut()) {
                (true, Some((_, entry))) => {
                    entry.push('\n');
                    entry.push_str(line);
                }
                _ => entries.push((line.to_string(), line.to_string())),
            }
        }
        entries
    }

    // The file's items, in the order they appear, each on one line.
    fn extract_items(language: &Language, query: &Query, source: &str) -> ApplicationResult<Vec<String>> {
        let Some(tree) = TreeSitterQueryRunner::parse(language, source)? else {
            return Ok(Vec::new());
        };
        let tree: &Tree = &tree;
        let captured_nodes: Vec<CapturedNode> = TreeSitterQueryRunner::capture(query, tree, source);

        let mut items: Vec<Range<usize>> = captured_nodes
            .iter()
            .filter(|captured_node: &&CapturedNode| captured_node.capture_name == ITEM_CAPTURE)
            .map(|captured_node: &CapturedNode| captured_node.byte_range.clone())
            .collect();
        items.sort_by_key(|item: &Range<usize>| (item.start, item.end));
        items.dedup();
        let mut left_out: Vec<Range<usize>> = captured_nodes
            .iter()
            .filter(|captured_node: &&CapturedNode| LEFT_OUT_CAPTURES.contains(&captured_node.capture_name.as_str()))
            .map(|captured_node: &CapturedNode| captured_node.byte_range.clone())
            .collect();
        left_out.sort_by_key(|range: &Range<usize>| (range.start, range.end));

        Ok(items
            .iter()
            .map(|item: &Range<usize>| {
                ProjectMapProcessor::to_one_line(&ProjectMapProcessor::without(source, item, &left_out))
            })
            .collect())
    }

    // The item's text without the left-out ranges inside it (sorted by start; one inside another is skipped). A range
    // that is the whole item is another item's body that holds only this one (a Python class with a single field), so
    // it isn't left out of this item.
    fn without(source: &str, item: &Range<usize>, left_out: &[Range<usize>]) -> String {
        let mut text: String = String::new();
        let mut cursor: usize = item.start;
        for range in left_out {
            if range.start >= cursor && range.end <= item.end && range != item {
                text.push_str(&source[cursor..range.start]);
                cursor = range.end;
            }
        }
        text.push_str(&source[cursor..item.end]);
        text
    }

    // Runs of whitespace (a multi-line signature, a struct's fields) become one space.
    fn to_one_line(text: &str) -> String {
        text.split_whitespace().collect::<Vec<&str>>().join(" ")
    }
}
