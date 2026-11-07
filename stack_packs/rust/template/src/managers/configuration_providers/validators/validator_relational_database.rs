use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::relational_database_config::RelationalDatabaseConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ValidatorRelationalDatabase;

impl ValidatorRelationalDatabase {
    pub fn validate_relational_database_config(config: &RelationalDatabaseConfig) -> ApplicationResult<()> {
        let errors: Vec<String> = [("connection_string", config.connection_string.as_deref())]
            .into_iter()
            .filter_map(|(property_name, property_value): (&str, Option<&str>)| {
                ValidatorString::validate(property_name, property_value)
            })
            .collect();

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "Relational database configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "relational_database")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
