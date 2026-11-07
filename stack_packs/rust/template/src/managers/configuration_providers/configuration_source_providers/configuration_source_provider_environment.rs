use crate::managers::configuration_providers::config_models::document_database_config::DocumentDatabaseConfig;
use crate::managers::configuration_providers::config_models::example_config::ExampleConfig;
use crate::managers::configuration_providers::config_models::json_placeholder_config::JsonPlaceholderConfig;
use crate::managers::configuration_providers::config_models::relational_database_config::RelationalDatabaseConfig;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

pub struct ConfigurationSourceProviderEnvironment;

impl ConfigurationSourceProviderProtocol for ConfigurationSourceProviderEnvironment {
    fn get_example_config(&self) -> ApplicationResult<ExampleConfig> {
        Ok(ExampleConfig {
            greeting_prefix: Some(ConfigurationSourceProviderEnvironment::get_string(
                "EXAMPLE_APP_GREETING_PREFIX",
                "Hello",
            )),
        })
    }

    fn get_relational_database_config(&self) -> ApplicationResult<RelationalDatabaseConfig> {
        Ok(RelationalDatabaseConfig {
            // The app's database on the factory's development Postgres, which `dbmate up` creates and migrates.
            connection_string: Some(ConfigurationSourceProviderEnvironment::get_string(
                "EXAMPLE_APP_RELATIONAL_DATABASE_CONNECTION_STRING",
                "postgresql://previews:previews@localhost:5433/example_app?sslmode=disable",
            )),
        })
    }

    fn get_document_database_config(&self) -> ApplicationResult<DocumentDatabaseConfig> {
        Ok(DocumentDatabaseConfig {
            connection_string: Some(ConfigurationSourceProviderEnvironment::get_string(
                "EXAMPLE_APP_DOCUMENT_DATABASE_CONNECTION_STRING",
                "in-memory://document",
            )),
        })
    }

    fn get_json_placeholder_config(&self) -> ApplicationResult<JsonPlaceholderConfig> {
        Ok(JsonPlaceholderConfig {
            base_url: Some(ConfigurationSourceProviderEnvironment::get_string(
                "EXAMPLE_APP_JSON_PLACEHOLDER_BASE_URL",
                "https://jsonplaceholder.typicode.com",
            )),
            timeout_seconds: Some(ConfigurationSourceProviderEnvironment::get_float(
                "EXAMPLE_APP_JSON_PLACEHOLDER_TIMEOUT_SECONDS",
                "10",
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
