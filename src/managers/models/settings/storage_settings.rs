use std::path::PathBuf;

use crate::managers::models::settings::relational_database_settings::RelationalDatabaseSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageSettings {
    /// Where the session store keeps each session's folder.
    pub sessions_directory: PathBuf,
    /// Where the lock store keeps the locks that keep two processes from running in one app, or one session, at once.
    pub locks_directory: PathBuf,
    /// Where the stack packs are.
    pub stack_packs_directory: PathBuf,
    /// Where apps started by name are kept, each in a folder named after it (its kebab-case name). A client that
    /// addresses apps by name never names a path.
    pub workspaces_directory: PathBuf,
    /// The Postgres database that keeps the sessions and their locks, when one is configured; without one they are
    /// kept in `sessions_directory` and `locks_directory`.
    pub relational_database: Option<RelationalDatabaseSettings>,
}
