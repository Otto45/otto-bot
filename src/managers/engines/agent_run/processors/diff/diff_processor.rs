use std::collections::{BTreeMap, BTreeSet};

use similar::TextDiff;

use crate::managers::models::workspace_file::WorkspaceFile;

// A review reads the diff whole: past this, it is cut short, and the reviewer reads the rest of what it needs.
const MAX_DIFF_CHARS: usize = 150_000;
const CONTEXT_LINES: usize = 3;

// What a change did to the workspace, as a unified diff, worked out from two snapshots of its files: before the change
// and after it.
pub struct DiffProcessor;

impl DiffProcessor {
    /// Every file that differs between the snapshots, in path order, created and deleted ones included (against
    /// /dev/null); empty when nothing changed.
    pub fn diff(&self, before: &[WorkspaceFile], after: &[WorkspaceFile]) -> String {
        let before_contents: BTreeMap<&str, &str> = DiffProcessor::to_contents(before);
        let after_contents: BTreeMap<&str, &str> = DiffProcessor::to_contents(after);
        let paths: BTreeSet<&str> = before_contents.keys().chain(after_contents.keys()).copied().collect();

        let mut diff: String = String::new();
        for path in paths {
            let old: Option<&str> = before_contents.get(path).copied();
            let new: Option<&str> = after_contents.get(path).copied();
            if old == new {
                continue;
            }
            let old_header: String = old.map_or_else(|| "/dev/null".to_string(), |_: &str| format!("a/{path}"));
            let new_header: String = new.map_or_else(|| "/dev/null".to_string(), |_: &str| format!("b/{path}"));
            let text_diff: TextDiff<'_, '_, '_, str> =
                TextDiff::from_lines(old.unwrap_or_default(), new.unwrap_or_default());
            diff.push_str(
                &text_diff
                    .unified_diff()
                    .context_radius(CONTEXT_LINES)
                    .header(&old_header, &new_header)
                    .to_string(),
            );
            if !diff.ends_with('\n') {
                diff.push('\n');
            }
        }

        let char_count: usize = diff.chars().count();
        if char_count <= MAX_DIFF_CHARS {
            return diff;
        }
        let kept: String = diff.chars().take(MAX_DIFF_CHARS).collect();
        format!(
            "{kept}\n[... the diff goes on for {} more characters]\n",
            char_count - MAX_DIFF_CHARS
        )
    }

    fn to_contents(workspace_files: &[WorkspaceFile]) -> BTreeMap<&str, &str> {
        workspace_files
            .iter()
            .map(|workspace_file: &WorkspaceFile| (workspace_file.path.as_str(), workspace_file.content.as_str()))
            .collect()
    }
}
