/// The AgentRunEngine's exceptions (its own and its processors'): one family of ExceptionKind. InvalidToolCall and
/// InvalidPatch are told to the model as its tool call's output, so it can correct the call; they never end a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunEngineException {
    InvalidToolCall,
    InvalidPatch,
    /// A tool the agent's role may not call: told to the model, which was told which it may.
    ToolNotAllowed,
    /// A work plan that fails the harness's checks: told to the model, which fixes the plan and submits it again.
    InvalidBrief,
    MaxTurnsReached,
}

impl AgentRunEngineException {
    pub fn name(self) -> &'static str {
        match self {
            AgentRunEngineException::InvalidToolCall => "InvalidToolCallException",
            AgentRunEngineException::InvalidPatch => "InvalidPatchException",
            AgentRunEngineException::ToolNotAllowed => "ToolNotAllowedException",
            AgentRunEngineException::InvalidBrief => "InvalidBriefException",
            AgentRunEngineException::MaxTurnsReached => "MaxTurnsReachedException",
        }
    }
}
