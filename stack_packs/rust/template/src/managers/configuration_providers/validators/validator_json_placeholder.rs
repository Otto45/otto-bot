use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::json_placeholder_config::JsonPlaceholderConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ValidatorJsonPlaceholder;

impl ValidatorJsonPlaceholder {
    pub fn validate_json_placeholder_config(config: &JsonPlaceholderConfig) -> ApplicationResult<()> {
        let mut errors: Vec<String> = Vec::new();

        if let Some(base_url_error) = ValidatorString::validate("base_url", config.base_url.as_deref()) {
            errors.push(base_url_error);
        }

        if config.timeout_seconds.is_none() {
            errors.push("timeout_seconds is missing".to_string());
        }

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "JSON Placeholder configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "json_placeholder")
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
                    "JSON Placeholder configuration is invalid",
                    ContextualData::new()
                        .with("setting_name", "json_placeholder")
                        .with("errors", error),
                ));
            }
        }
        Ok(())
    }
}
