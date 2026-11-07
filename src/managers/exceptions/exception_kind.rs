use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::command_runner_gateway_exception::CommandRunnerGatewayException;
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::exceptions::otto_bot_manager_exception::OttoBotManagerException;
use crate::managers::exceptions::session_store_gateway_exception::SessionStoreGatewayException;
use crate::managers::exceptions::stack_pack_store_gateway_exception::StackPackStoreGatewayException;
use crate::managers::exceptions::workspace_gateway_exception::WorkspaceGatewayException;

/// Which of the application's exceptions one is. Each component's exceptions are a family: an enum nested in this
/// one. Code that treats a family alike matches the family (`ExceptionKind::WorkspaceGateway(_)`), and a match that
/// names every family fails to compile when a new one is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    AgentRunEngine(AgentRunEngineException),
    CommandRunnerGateway(CommandRunnerGatewayException),
    ConfigurationProvider(ConfigurationProviderException),
    OpenAiResponsesGateway(OpenAiResponsesGatewayException),
    OttoBotManager(OttoBotManagerException),
    SessionStoreGateway(SessionStoreGatewayException),
    StackPackStoreGateway(StackPackStoreGatewayException),
    WorkspaceGateway(WorkspaceGatewayException),
    // Raised for something the client can put right, so its message is written for the person using the app: a
    // service answers with it as it is. What only engineers need goes in the context, which is logged.
    InvalidUserRequest,
}

impl ExceptionKind {
    /// The name the exception is logged under (e.g. "SessionNotFoundException").
    pub fn name(self) -> &'static str {
        match self {
            ExceptionKind::AgentRunEngine(exception) => exception.name(),
            ExceptionKind::CommandRunnerGateway(exception) => exception.name(),
            ExceptionKind::ConfigurationProvider(exception) => exception.name(),
            ExceptionKind::OpenAiResponsesGateway(exception) => exception.name(),
            ExceptionKind::OttoBotManager(exception) => exception.name(),
            ExceptionKind::SessionStoreGateway(exception) => exception.name(),
            ExceptionKind::StackPackStoreGateway(exception) => exception.name(),
            ExceptionKind::WorkspaceGateway(exception) => exception.name(),
            ExceptionKind::InvalidUserRequest => "InvalidUserRequestException",
        }
    }
}

impl From<AgentRunEngineException> for ExceptionKind {
    fn from(exception: AgentRunEngineException) -> Self {
        ExceptionKind::AgentRunEngine(exception)
    }
}

impl From<CommandRunnerGatewayException> for ExceptionKind {
    fn from(exception: CommandRunnerGatewayException) -> Self {
        ExceptionKind::CommandRunnerGateway(exception)
    }
}

impl From<ConfigurationProviderException> for ExceptionKind {
    fn from(exception: ConfigurationProviderException) -> Self {
        ExceptionKind::ConfigurationProvider(exception)
    }
}

impl From<OpenAiResponsesGatewayException> for ExceptionKind {
    fn from(exception: OpenAiResponsesGatewayException) -> Self {
        ExceptionKind::OpenAiResponsesGateway(exception)
    }
}

impl From<OttoBotManagerException> for ExceptionKind {
    fn from(exception: OttoBotManagerException) -> Self {
        ExceptionKind::OttoBotManager(exception)
    }
}

impl From<SessionStoreGatewayException> for ExceptionKind {
    fn from(exception: SessionStoreGatewayException) -> Self {
        ExceptionKind::SessionStoreGateway(exception)
    }
}

impl From<StackPackStoreGatewayException> for ExceptionKind {
    fn from(exception: StackPackStoreGatewayException) -> Self {
        ExceptionKind::StackPackStoreGateway(exception)
    }
}

impl From<WorkspaceGatewayException> for ExceptionKind {
    fn from(exception: WorkspaceGatewayException) -> Self {
        ExceptionKind::WorkspaceGateway(exception)
    }
}
