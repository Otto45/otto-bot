/// A file a scaffold creates: its path relative to the workspace, and its content (a template until rendered).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldFile {
    pub path: String,
    pub content: String,
}
