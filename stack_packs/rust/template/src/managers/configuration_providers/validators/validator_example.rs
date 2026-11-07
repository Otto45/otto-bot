use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::example_config::ExampleConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ValidatorExample;

impl ValidatorExample {
    pub fn validate_example_config(config: &ExampleConfig) -> ApplicationResult<()> {
        let errors: Vec<String> = [("greeting_prefix", config.greeting_prefix.as_deref())]
            .into_iter()
            .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
                ValidatorString::validate(property_name, property_value)
            })
            .collect();

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "Example configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "example")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
