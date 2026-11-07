use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    /// Where the session store keeps each session's folder.
    pub sessions_directory: PathBuf,
    /// Where the stack packs are.
    pub stack_packs_directory: PathBuf,
}
