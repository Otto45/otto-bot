/// A file of the workspace and its text, by its path relative to the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
}
