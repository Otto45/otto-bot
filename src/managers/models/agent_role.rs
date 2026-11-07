/// What an agent is for. Every role gets the same tools in the same order (so the cached prefix is shared); a role is
/// restricted by which of them it may call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    /// The session's agent: does a small job itself, and plans, skeletons and integrates a bigger one.
    Lead,
    /// Builds one work package, patching only the files it owns.
    Worker,
    /// Reads the change and reports what is wrong with it; changes nothing.
    Reviewer,
}

impl AgentRole {
    pub fn name(self) -> &'static str {
        match self {
            AgentRole::Lead => "lead",
            AgentRole::Worker => "worker",
            AgentRole::Reviewer => "reviewer",
        }
    }
}
