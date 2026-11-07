use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::document_database_gateway_exception::DocumentDatabaseGatewayException;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

/// Which of the application's exceptions one is. Each component's exceptions are a family: an enum nested in this
/// one. Code that treats a family alike matches the family (`ExceptionKind::JsonPlaceholderGateway(_)`), and a match
/// that names every family fails to compile when a new one is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    ConfigurationProvider(ConfigurationProviderException),
    RelationalDatabaseGateway(RelationalDatabaseGatewayException),
    DocumentDatabaseGateway(DocumentDatabaseGatewayException),
    JsonPlaceholderGateway(JsonPlaceholderGatewayException),
    // Raised for something the caller can put right, so its message is written for the person using the app: a
    // service answers with it as it is. What only engineers need goes in the context, which is logged.
    InvalidCustomer,
}

impl ExceptionKind {
    /// The name the exception is logged under (e.g. "CustomerNotFoundException").
    pub fn name(self) -> &'static str {
        match self {
            ExceptionKind::ConfigurationProvider(exception) => exception.name(),
            ExceptionKind::RelationalDatabaseGateway(exception) => exception.name(),
            ExceptionKind::DocumentDatabaseGateway(exception) => exception.name(),
            ExceptionKind::JsonPlaceholderGateway(exception) => exception.name(),
            ExceptionKind::InvalidCustomer => "InvalidCustomerException",
        }
    }
}

impl From<ConfigurationProviderException> for ExceptionKind {
    fn from(exception: ConfigurationProviderException) -> Self {
        ExceptionKind::ConfigurationProvider(exception)
    }
}

impl From<RelationalDatabaseGatewayException> for ExceptionKind {
    fn from(exception: RelationalDatabaseGatewayException) -> Self {
        ExceptionKind::RelationalDatabaseGateway(exception)
    }
}

impl From<DocumentDatabaseGatewayException> for ExceptionKind {
    fn from(exception: DocumentDatabaseGatewayException) -> Self {
        ExceptionKind::DocumentDatabaseGateway(exception)
    }
}

impl From<JsonPlaceholderGatewayException> for ExceptionKind {
    fn from(exception: JsonPlaceholderGatewayException) -> Self {
        ExceptionKind::JsonPlaceholderGateway(exception)
    }
}
