use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::exceptions::otto_bot_manager_exception::OttoBotManagerException;

/// Which of the application's exceptions one is. Each component's exceptions are a family: an enum nested in this
/// one. Code that treats a family alike matches the family (`ExceptionKind::OpenAiResponsesGateway(_)`), and a match
/// that names every family fails to compile when a new one is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    ConfigurationProvider(ConfigurationProviderException),
    OpenAiResponsesGateway(OpenAiResponsesGatewayException),
    OttoBotManager(OttoBotManagerException),
    // Raised for something the client can put right, so its message is written for the person using the app: a
    // service answers with it as it is. What only engineers need goes in the context, which is logged.
    InvalidUserRequest,
}

impl ExceptionKind {
    /// The name the exception is logged under (e.g. "SessionNotFoundException").
    pub fn name(self) -> &'static str {
        match self {
            ExceptionKind::ConfigurationProvider(exception) => exception.name(),
            ExceptionKind::OpenAiResponsesGateway(exception) => exception.name(),
            ExceptionKind::OttoBotManager(exception) => exception.name(),
            ExceptionKind::InvalidUserRequest => "InvalidUserRequestException",
        }
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
