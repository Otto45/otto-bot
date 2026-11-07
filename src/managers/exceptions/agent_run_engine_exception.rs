/// The AgentRunEngine's exceptions (its own and its processors'): one family of ExceptionKind. InvalidToolCall and
/// InvalidPatch are told to the model as its tool call's output, so it can correct the call; they never end a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunEngineException {
    InvalidToolCall,
    InvalidPatch,
    MaxTurnsReached,
    /// The workspace's stack pack maps projects in a way the harness can't: an unknown grammar, a query that doesn't
    /// compile, a glob that isn't valid.
    InvalidProjectMap,
}

impl AgentRunEngineException {
    pub fn name(self) -> &'static str {
        match self {
            AgentRunEngineException::InvalidToolCall => "InvalidToolCallException",
            AgentRunEngineException::InvalidPatch => "InvalidPatchException",
            AgentRunEngineException::MaxTurnsReached => "MaxTurnsReachedException",
            AgentRunEngineException::InvalidProjectMap => "InvalidProjectMapException",
        }
    }
}
