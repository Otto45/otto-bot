#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StorageConfig {
    /// otto-bot's own folder: sessions (and the CLI's log) are kept under it.
    pub home_directory: Option<String>,
    pub stack_packs_directory: Option<String>,
}
