use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ConfigurationSourceProviderEnvironment;

impl ConfigurationSourceProviderProtocol for ConfigurationSourceProviderEnvironment {
    fn get_agent_config(&self) -> ApplicationResult<AgentConfig> {
        Ok(AgentConfig {
            default_service_tier: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_SERVICE_TIER",
                "standard",
            )),
        })
    }

    fn get_openai_responses_config(&self) -> ApplicationResult<OpenAiResponsesConfig> {
        Ok(OpenAiResponsesConfig {
            base_url: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_OPENAI_BASE_URL",
                "https://api.openai.com/v1",
            )),
            // The name every OpenAI client reads, so the sandbox VM sets one variable for all of them. It has no
            // default: without it, creating the facade fails.
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_MODEL",
                "gpt-6.1-sol",
            )),
            // Long: a response that reasons at high effort can take minutes.
            timeout_seconds: Some(ConfigurationSourceProviderEnvironment::get_float(
                "OTTO_BOT_OPENAI_TIMEOUT_SECONDS",
                "600",
            )?),
        })
    }
}

impl ConfigurationSourceProviderEnvironment {
    fn get_string(environment_variable_name: &str, default_value: &str) -> String {
        std::env::var(environment_variable_name).unwrap_or_else(|_: std::env::VarError| default_value.to_string())
    }

    fn get_float(environment_variable_name: &str, default_value: &str) -> ApplicationResult<f64> {
        let value: String =
            ConfigurationSourceProviderEnvironment::get_string(environment_variable_name, default_value);
        value
            .trim()
            .parse::<f64>()
            .map_err(|exception: std::num::ParseFloatError| {
                ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "Environment variable is not a number",
                    ContextualData::new().with("environment_variable_name", environment_variable_name),
                )
                .with_source(exception)
            })
    }
}
