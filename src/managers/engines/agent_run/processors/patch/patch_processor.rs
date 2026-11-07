use crate::managers::engines::agent_run::processors::patch::diff_appliers::v4a_diff_applier::V4aDiffApplier;
use crate::managers::exceptions::application_base_exception::ApplicationResult;

// Turns the model's apply_patch operations into files' contents. Reading and writing the files is the workspace's.
pub struct PatchProcessor;

impl PatchProcessor {
    /// A new file's content, from a create_file operation's diff.
    pub fn create_file_content(&self, diff: &str) -> ApplicationResult<String> {
        V4aDiffApplier::apply_create(diff)
    }

    /// A file's new content, from its current content and an update_file operation's diff.
    pub fn update_file_content(&self, content: &str, diff: &str) -> ApplicationResult<String> {
        V4aDiffApplier::apply_update(content, diff)
    }
}
