use std::sync::Arc;
use std::time::Duration;

use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_environment::ConfigurationSourceProviderEnvironment;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connector::ResponsesWebSocketConnector;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;
use crate::managers::service_locator::service_locator_seams_protocol::ServiceLocatorSeamsProtocol;

/// The production seams: the environment and the network.
pub struct ServiceLocatorSeams;

impl ServiceLocatorSeamsProtocol for ServiceLocatorSeams {
    fn create_configuration_source_provider(&self) -> Arc<dyn ConfigurationSourceProviderProtocol> {
        Arc::new(ConfigurationSourceProviderEnvironment)
    }

    fn create_openai_responses_http_client(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<reqwest::Client> {
        let timeout: Duration = ServiceLocatorSeams::to_timeout(openai_responses_settings)?;

        reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|exception: reqwest::Error| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiRequest,
                    "OpenAI HTTP client could not be built",
                    ContextualData::new(),
                )
                .with_source(exception)
            })
    }

    fn create_openai_responses_web_socket_connector(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<ResponsesWebSocketConnector> {
        Ok(ResponsesWebSocketConnector::new(
            &openai_responses_settings.base_url,
            openai_responses_settings.api_key.clone(),
            ServiceLocatorSeams::to_timeout(openai_responses_settings)?,
        ))
    }
}

impl ServiceLocatorSeams {
    // The validator keeps the timeout positive and finite; one too long for a Duration is refused here.
    fn to_timeout(openai_responses_settings: &OpenAiResponsesSettings) -> ApplicationResult<Duration> {
        Duration::try_from_secs_f64(openai_responses_settings.timeout_seconds).map_err(
            |_: std::time::TryFromFloatSecsError| {
                ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "OpenAI configuration is invalid",
                    ContextualData::new()
                        .with("setting_name", "openai_responses")
                        .with("errors", "timeout_seconds is too long"),
                )
            },
        )
    }
}
