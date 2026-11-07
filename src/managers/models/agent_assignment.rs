use std::path::PathBuf;

use crate::managers::models::agent_role::AgentRole;

/// One agent's part in a session's work: the lane it converses on, its role, the workspace it works in (the session's,
/// or a worker's copy of it) and the files it may change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAssignment {
    /// Unique within the session, and snake_case: the lead's is "lead", a worker's is its package's name.
    pub lane_name: String,
    pub role: AgentRole,
    /// Absolute and canonical: every path the agent uses is relative to it.
    pub workspace_directory: PathBuf,
    /// The only files it may create, change or delete; none for an agent that may change any file.
    pub owned_paths: Option<Vec<String>>,
}
