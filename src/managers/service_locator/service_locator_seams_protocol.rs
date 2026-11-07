use std::sync::Arc;

use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connector::ResponsesWebSocketConnector;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;

/// The ServiceLocator's `create_*` seams: the objects that reach outside the process, built from settings the
/// locator hands in. The locator is given its seams, so whatever needs others builds the same production
/// ServiceLocator with seams of its own, which call ServiceLocatorSeams for every seam they keep.
pub trait ServiceLocatorSeamsProtocol: Send + Sync {
    fn create_configuration_source_provider(&self) -> Arc<dyn ConfigurationSourceProviderProtocol>;

    fn create_openai_responses_http_client(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<reqwest::Client>;

    fn create_openai_responses_web_socket_connector(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<ResponsesWebSocketConnector>;
}
