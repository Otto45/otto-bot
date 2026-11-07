use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::storage_config::StorageConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ValidatorStorage;

impl ValidatorStorage {
    pub fn validate_storage_config(config: &StorageConfig) -> ApplicationResult<()> {
        let errors: Vec<String> = [
            (
                "home_directory (OTTO_BOT_HOME, or HOME)",
                config.home_directory.as_deref(),
            ),
            ("stack_packs_directory", config.stack_packs_directory.as_deref()),
        ]
        .into_iter()
        .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
            ValidatorString::validate(property_name, property_value)
        })
        .collect();

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "Storage configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "storage")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
