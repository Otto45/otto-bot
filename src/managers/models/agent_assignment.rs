use std::path::PathBuf;

use crate::managers::models::agent_role::AgentRole;

/// One agent's part in a session's work: the lane it converses on, its role, and the workspace it works in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAssignment {
    /// Unique within the session, and snake_case: each role's lane is named after it ("orchestrator", "implementor").
    pub lane_name: String,
    pub role: AgentRole,
    /// Absolute and canonical: every path the agent uses is relative to it.
    pub workspace_directory: PathBuf,
}
