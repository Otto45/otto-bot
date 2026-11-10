use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    /// Where the session store keeps each session's folder.
    pub sessions_directory: PathBuf,
    /// Where the stack packs are.
    pub stack_packs_directory: PathBuf,
    /// Where apps started by name are kept, each in a folder named after it (its kebab-case name). A client that
    /// addresses apps by name never names a path.
    pub workspaces_directory: PathBuf,
}
