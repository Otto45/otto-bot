/// The CommandRunnerGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandRunnerGatewayException {
    /// The command could not be started (e.g. its program is not installed).
    CommandStart,
}

impl CommandRunnerGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            CommandRunnerGatewayException::CommandStart => "CommandStartException",
        }
    }
}
