/// What an agent is for. Every role gets the same tools in the same order (so the cached prefix is shared); a role is
/// restricted by which of them it may call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRole {
    /// The session's agent: makes a small change itself, and plans a bigger one for the implementor.
    Orchestrator,
    /// Builds the orchestrator's plan: the only role that changes files under a plan.
    Implementor,
    /// Reads the change and reports what is wrong with it; changes nothing.
    Reviewer,
}

impl AgentRole {
    pub fn name(self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "orchestrator",
            AgentRole::Implementor => "implementor",
            AgentRole::Reviewer => "reviewer",
        }
    }
}
