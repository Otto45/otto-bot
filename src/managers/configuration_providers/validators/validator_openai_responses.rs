use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::openai_transport::OpenAiTransport;

pub struct ValidatorOpenAiResponses;

impl ValidatorOpenAiResponses {
    pub fn validate_openai_responses_config(config: &OpenAiResponsesConfig) -> ApplicationResult<()> {
        let mut errors: Vec<String> = [
            ("base_url", config.base_url.as_deref()),
            ("api_key (OPENAI_API_KEY)", config.api_key.as_deref()),
            ("model", config.model.as_deref()),
            ("transport", config.transport.as_deref()),
        ]
        .into_iter()
        .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
            ValidatorString::validate(property_name, property_value)
        })
        .collect();

        if config.timeout_seconds.is_none() {
            errors.push("timeout_seconds is missing".to_string());
        }
        if config.idle_timeout_seconds.is_none() {
            errors.push("idle_timeout_seconds is missing".to_string());
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

        if let Some(transport) = config.transport.as_deref()
            && OpenAiTransport::from_name(transport).is_none()
        {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::InvalidConfiguration,
                "OpenAI configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "openai_responses")
                    .with("errors", "transport is not one of websocket, http"),
            ));
        }

        for (setting_name, seconds) in [
            ("timeout_seconds", config.timeout_seconds),
            ("idle_timeout_seconds", config.idle_timeout_seconds),
        ] {
            let Some(seconds) = seconds else {
                continue;
            };
            // NaN is not positive either; an infinite timeout is no timeout.
            let error: Option<String> = if seconds.is_nan() || seconds <= 0.0 {
                Some(format!("{setting_name} is not positive"))
            } else if seconds.is_infinite() {
                Some(format!("{setting_name} is not finite"))
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
