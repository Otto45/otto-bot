use std::sync::Arc;

use crate::managers::configuration_providers::config_models::document_database_config::DocumentDatabaseConfig;
use crate::managers::configuration_providers::config_models::example_config::ExampleConfig;
use crate::managers::configuration_providers::config_models::json_placeholder_config::JsonPlaceholderConfig;
use crate::managers::configuration_providers::config_models::relational_database_config::RelationalDatabaseConfig;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::configuration_providers::validators::validator_document_database::ValidatorDocumentDatabase;
use crate::managers::configuration_providers::validators::validator_example::ValidatorExample;
use crate::managers::configuration_providers::validators::validator_json_placeholder::ValidatorJsonPlaceholder;
use crate::managers::configuration_providers::validators::validator_relational_database::ValidatorRelationalDatabase;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::settings::document_database_settings::DocumentDatabaseSettings;
use crate::managers::models::settings::example_settings::ExampleSettings;
use crate::managers::models::settings::json_placeholder_settings::JsonPlaceholderSettings;
use crate::managers::models::settings::relational_database_settings::RelationalDatabaseSettings;

/// Loads, validates and maps every setting on construction, so malformed configuration fails the moment the
/// provider is created, before anything that depends on it is built.
pub struct ConfigurationProvider {
    example_settings: ExampleSettings,
    relational_database_settings: RelationalDatabaseSettings,
    document_database_settings: DocumentDatabaseSettings,
    json_placeholder_settings: JsonPlaceholderSettings,
}

impl ConfigurationProvider {
    /// Requires its source: the ServiceLocator is the only place that decides where configuration comes from.
    pub fn new(configuration_source_provider: Arc<dyn ConfigurationSourceProviderProtocol>) -> ApplicationResult<Self> {
        let source_provider: &dyn ConfigurationSourceProviderProtocol = configuration_source_provider.as_ref();

        Ok(ConfigurationProvider {
            example_settings: ConfigurationProvider::load_example_settings(source_provider)?,
            relational_database_settings: ConfigurationProvider::load_relational_database_settings(source_provider)?,
            document_database_settings: ConfigurationProvider::load_document_database_settings(source_provider)?,
            json_placeholder_settings: ConfigurationProvider::load_json_placeholder_settings(source_provider)?,
        })
    }

    pub fn get_example_settings(&self) -> &ExampleSettings {
        &self.example_settings
    }

    pub fn get_relational_database_settings(&self) -> &RelationalDatabaseSettings {
        &self.relational_database_settings
    }

    pub fn get_document_database_settings(&self) -> &DocumentDatabaseSettings {
        &self.document_database_settings
    }

    pub fn get_json_placeholder_settings(&self) -> &JsonPlaceholderSettings {
        &self.json_placeholder_settings
    }

    // Each loader maps a config only after its validator passed it, so each `expect` states a checked fact: it
    // cannot fail unless the validator is wrong.
    fn load_example_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<ExampleSettings> {
        let config: ExampleConfig = source_provider.get_example_config()?;
        ValidatorExample::validate_example_config(&config)?;
        Ok(ExampleSettings {
            greeting_prefix: config.greeting_prefix.expect("validated by ValidatorExample"),
        })
    }

    fn load_relational_database_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<RelationalDatabaseSettings> {
        let config: RelationalDatabaseConfig = source_provider.get_relational_database_config()?;
        ValidatorRelationalDatabase::validate_relational_database_config(&config)?;
        Ok(RelationalDatabaseSettings {
            connection_string: config
                .connection_string
                .expect("validated by ValidatorRelationalDatabase"),
        })
    }

    fn load_document_database_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<DocumentDatabaseSettings> {
        let config: DocumentDatabaseConfig = source_provider.get_document_database_config()?;
        ValidatorDocumentDatabase::validate_document_database_config(&config)?;
        Ok(DocumentDatabaseSettings {
            connection_string: config
                .connection_string
                .expect("validated by ValidatorDocumentDatabase"),
        })
    }

    fn load_json_placeholder_settings(
        source_provider: &dyn ConfigurationSourceProviderProtocol,
    ) -> ApplicationResult<JsonPlaceholderSettings> {
        let config: JsonPlaceholderConfig = source_provider.get_json_placeholder_config()?;
        ValidatorJsonPlaceholder::validate_json_placeholder_config(&config)?;
        Ok(JsonPlaceholderSettings {
            base_url: config.base_url.expect("validated by ValidatorJsonPlaceholder"),
            timeout_seconds: config.timeout_seconds.expect("validated by ValidatorJsonPlaceholder"),
            max_retries: config.max_retries.expect("validated by ValidatorJsonPlaceholder"),
        })
    }
}
