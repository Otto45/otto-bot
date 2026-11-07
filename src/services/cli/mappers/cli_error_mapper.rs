use crate::managers::exceptions::exception_kind::ExceptionKind;

// Decides, for every exception, whether the person using the CLI can put it right.
pub struct CliErrorMapper;

impl CliErrorMapper {
    /// Names every family, with no `_` arm, so a new family doesn't compile until it is decided here.
    pub fn is_client_error(exception_kind: ExceptionKind) -> bool {
        match exception_kind {
            ExceptionKind::OttoBotManager(_) | ExceptionKind::InvalidUserRequest => true,
            // The person running the CLI is who sets its configuration (e.g. OPENAI_API_KEY), so they can fix it.
            ExceptionKind::ConfigurationProvider(_) => true,
            // Reaches the CLI only for the workspace it was given (e.g. a --workspace that doesn't exist); the agent's
            // own workspace errors are told to the model.
            ExceptionKind::WorkspaceGateway(_) => true,
            ExceptionKind::AgentRunEngine(_)
            | ExceptionKind::CommandRunnerGateway(_)
            | ExceptionKind::OpenAiResponsesGateway(_)
            | ExceptionKind::SessionStoreGateway(_)
            | ExceptionKind::StackPackStoreGateway(_) => false,
        }
    }
}
