use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ValidatorOpenAiResponses;

impl ValidatorOpenAiResponses {
    pub fn validate_openai_responses_config(config: &OpenAiResponsesConfig) -> ApplicationResult<()> {
        let mut errors: Vec<String> = [
            ("base_url", config.base_url.as_deref()),
            ("api_key (OPENAI_API_KEY)", config.api_key.as_deref()),
            ("model", config.model.as_deref()),
        ]
        .into_iter()
        .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
            ValidatorString::validate(property_name, property_value)
        })
        .collect();

        if config.timeout_seconds.is_none() {
            errors.push("timeout_seconds is missing".to_string());
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "OpenAI configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "openai_responses")
                    .with("errors", errors.join("; ")),
            ));
        }

        if let Some(timeout_seconds) = config.timeout_seconds {
            // NaN is not positive either; an infinite timeout is no timeout.
            let error: Option<&str> = if timeout_seconds.is_nan() || timeout_seconds <= 0.0 {
                Some("timeout_seconds is not positive")
            } else if timeout_seconds.is_infinite() {
                Some("timeout_seconds is not finite")
            } else {
                None
            };
            if let Some(error) = error {
                return Err(ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "OpenAI configuration is invalid",
                    ContextualData::new()
                        .with("setting_name", "openai_responses")
                        .with("errors", error),
                ));
            }
        }
        Ok(())
    }
}
