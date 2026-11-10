#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageConfig {
    /// otto-bot's own folder: sessions (and the CLI's log) are kept under it.
    pub home_directory: Option<String>,
    pub stack_packs_directory: Option<String>,
    /// Where apps started by name are kept, each in a folder named after it; None for the home folder's apps/.
    pub workspaces_directory: Option<String>,
}
