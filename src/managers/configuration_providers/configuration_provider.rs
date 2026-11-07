use std::sync::Arc;

use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::configuration_providers::validators::validator_agent::ValidatorAgent;
use crate::managers::configuration_providers::validators::validator_openai_responses::ValidatorOpenAiResponses;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::settings::agent_settings::AgentSettings;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;

/// Loads, validates and maps every setting on construction, so malformed configuration fails the moment the
/// provider is created, before anything that depends on it is built.
pub struct ConfigurationProvider {
    agent_settings: AgentSettings,
    openai_responses_settings: OpenAiResponsesSettings,
}

impl ConfigurationProvider {
    /// Requires its source: the ServiceLocator is the only place that decides where configuration comes from.
    pub fn new(configuration_source_provider: Arc<dyn ConfigurationSourceProviderProtocol>) -> ApplicationResult<Self> {
        let source_provider: &dyn ConfigurationSourceProviderProtocol = configuration_source_provider.as_ref();

        Ok(ConfigurationProvider {
            agent_settings: ConfigurationProvider::load_agent_settings(source_provider)?,
            openai_responses_settings: ConfigurationProvider::load_openai_responses_settings(source_provider)?,
        })
    }

    pub fn get_agent_settings(&self) -> &AgentSettings {
        &self.agent_settings
    }

    pub fn get_openai_responses_settings(&self) -> &OpenAiResponsesSettings {
        &self.openai_responses_settings
    }

    // Each loader maps a config only after its validator passed it, so each `expect` states a checked fact: it
    // cannot fail unless the validator is wrong.
    fn load_agent_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<AgentSettings> {
        let config: AgentConfig = source_provider.get_agent_config()?;
        ValidatorAgent::validate_agent_config(&config)?;
        let default_service_tier: String = config.default_service_tier.expect("validated by ValidatorAgent");
        Ok(AgentSettings {
            default_service_tier: ServiceTier::from_name(&default_service_tier).expect("validated by ValidatorAgent"),
        })
    }

    fn load_openai_responses_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<OpenAiResponsesSettings> {
        let config: OpenAiResponsesConfig = source_provider.get_openai_responses_config()?;
        ValidatorOpenAiResponses::validate_openai_responses_config(&config)?;
        Ok(OpenAiResponsesSettings {
            base_url: config.base_url.expect("validated by ValidatorOpenAiResponses"),
            api_key: config.api_key.expect("validated by ValidatorOpenAiResponses"),
            model: config.model.expect("validated by ValidatorOpenAiResponses"),
            timeout_seconds: config.timeout_seconds.expect("validated by ValidatorOpenAiResponses"),
        })
    }
}
