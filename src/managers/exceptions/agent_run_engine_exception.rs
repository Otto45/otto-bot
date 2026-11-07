/// The AgentRunEngine's exceptions (its own and its processors'): one family of ExceptionKind. InvalidToolCall and
/// InvalidPatch are told to the model as its tool call's output, so it can correct the call; they never end a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunEngineException {
    InvalidToolCall,
    InvalidPatch,
    MaxTurnsReached,
}

impl AgentRunEngineException {
    pub fn name(self) -> &'static str {
        match self {
            AgentRunEngineException::InvalidToolCall => "InvalidToolCallException",
            AgentRunEngineException::InvalidPatch => "InvalidPatchException",
            AgentRunEngineException::MaxTurnsReached => "MaxTurnsReachedException",
        }
    }
}
