/// The AgentRunEngine's exceptions (its own and its processors'): one family of ExceptionKind. InvalidToolCall and
/// InvalidPatch are told to the model as its tool call's output, so it can correct the call; they never end a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunEngineException {
    InvalidToolCall,
    InvalidPatch,
    /// A scaffold that can't be applied to the workspace (a file it would create exists, an anchor is missing): told
    /// to the model, which writes the component by hand instead.
    ScaffoldNotApplicable,
    /// A tool the agent's role may not call: told to the model, which was told which it may.
    ToolNotAllowed,
    /// A change to a file the agent doesn't own: told to the model, which says what it needs instead.
    FileNotOwned,
    MaxTurnsReached,
}

impl AgentRunEngineException {
    pub fn name(self) -> &'static str {
        match self {
            AgentRunEngineException::InvalidToolCall => "InvalidToolCallException",
            AgentRunEngineException::InvalidPatch => "InvalidPatchException",
            AgentRunEngineException::ScaffoldNotApplicable => "ScaffoldNotApplicableException",
            AgentRunEngineException::ToolNotAllowed => "ToolNotAllowedException",
            AgentRunEngineException::FileNotOwned => "FileNotOwnedException",
            AgentRunEngineException::MaxTurnsReached => "MaxTurnsReachedException",
        }
    }
}
