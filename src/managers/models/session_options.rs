use std::path::PathBuf;

use crate::managers::models::service_tier::ServiceTier;

/// What a client asks of a new session. Whatever it leaves out comes from configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionOptions {
    /// The directory the agent works in: it reads and writes nothing outside it.
    pub workspace_directory: PathBuf,
    pub service_tier: Option<ServiceTier>,
}
