use crate::managers::exceptions::agent_run_engine_exception::AgentRunEngineException;
use crate::managers::exceptions::command_runner_gateway_exception::CommandRunnerGatewayException;
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::lock_store_gateway_exception::LockStoreGatewayException;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::exceptions::otto_bot_manager_exception::OttoBotManagerException;
use crate::managers::exceptions::project_bootstrap_engine_exception::ProjectBootstrapEngineException;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
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
    LockStoreGateway(LockStoreGatewayException),
    OpenAiResponsesGateway(OpenAiResponsesGatewayException),
    OttoBotManager(OttoBotManagerException),
    ProjectBootstrapEngine(ProjectBootstrapEngineException),
    RelationalDatabaseGateway(RelationalDatabaseGatewayException),
    SessionStoreGateway(SessionStoreGatewayException),
    StackPackStoreGateway(StackPackStoreGatewayException),
    WorkspaceGateway(WorkspaceGatewayException),
    // Raised for something the client can put right, so its message is written for the person using the app: a
    // service answers with it as it is. What only engineers need goes in the context, which is logged.
    InvalidUserRequest,
    // The workspace's stack pack can't be used: a grammar otto-bot doesn't have, a query that doesn't compile, a glob
    // or pattern that isn't valid. Raised by common helpers, so it belongs to no component's family.
    InvalidStackPack,
}

impl ExceptionKind {
    /// The name the exception is logged under (e.g. "SessionNotFoundException").
    pub fn name(self) -> &'static str {
        match self {
            ExceptionKind::AgentRunEngine(exception) => exception.name(),
            ExceptionKind::CommandRunnerGateway(exception) => exception.name(),
            ExceptionKind::ConfigurationProvider(exception) => exception.name(),
            ExceptionKind::LockStoreGateway(exception) => exception.name(),
            ExceptionKind::OpenAiResponsesGateway(exception) => exception.name(),
            ExceptionKind::OttoBotManager(exception) => exception.name(),
            ExceptionKind::ProjectBootstrapEngine(exception) => exception.name(),
            ExceptionKind::RelationalDatabaseGateway(exception) => exception.name(),
            ExceptionKind::SessionStoreGateway(exception) => exception.name(),
            ExceptionKind::StackPackStoreGateway(exception) => exception.name(),
            ExceptionKind::WorkspaceGateway(exception) => exception.name(),
            ExceptionKind::InvalidUserRequest => "InvalidUserRequestException",
            ExceptionKind::InvalidStackPack => "InvalidStackPackException",
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

impl From<LockStoreGatewayException> for ExceptionKind {
    fn from(exception: LockStoreGatewayException) -> Self {
        ExceptionKind::LockStoreGateway(exception)
    }
}

impl From<OpenAiResponsesGatewayException> for ExceptionKind {
    fn from(exception: OpenAiResponsesGatewayException) -> Self {
        ExceptionKind::OpenAiResponsesGateway(exception)
    }
}

impl From<ProjectBootstrapEngineException> for ExceptionKind {
    fn from(exception: ProjectBootstrapEngineException) -> Self {
        ExceptionKind::ProjectBootstrapEngine(exception)
    }
}

impl From<OttoBotManagerException> for ExceptionKind {
    fn from(exception: OttoBotManagerException) -> Self {
        ExceptionKind::OttoBotManager(exception)
    }
}

impl From<RelationalDatabaseGatewayException> for ExceptionKind {
    fn from(exception: RelationalDatabaseGatewayException) -> Self {
        ExceptionKind::RelationalDatabaseGateway(exception)
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
