use crate::managers::configuration_providers::config_models::document_database_config::DocumentDatabaseConfig;
use crate::managers::configuration_providers::config_models::example_config::ExampleConfig;
use crate::managers::configuration_providers::config_models::json_placeholder_config::JsonPlaceholderConfig;
use crate::managers::configuration_providers::config_models::relational_database_config::RelationalDatabaseConfig;
use crate::managers::exceptions::application_base_exception::ApplicationResult;

pub trait ConfigurationSourceProviderProtocol: Send + Sync {
    fn get_example_config(&self) -> ApplicationResult<ExampleConfig>;

    fn get_relational_database_config(&self) -> ApplicationResult<RelationalDatabaseConfig>;

    fn get_document_database_config(&self) -> ApplicationResult<DocumentDatabaseConfig>;

    fn get_json_placeholder_config(&self) -> ApplicationResult<JsonPlaceholderConfig>;
}
