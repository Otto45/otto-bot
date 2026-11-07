use std::sync::Arc;

use sqlx::PgPool;

use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::InMemoryDocumentDatabaseClient;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::settings::document_database_settings::DocumentDatabaseSettings;
use crate::managers::models::settings::json_placeholder_settings::JsonPlaceholderSettings;
use crate::managers::models::settings::relational_database_settings::RelationalDatabaseSettings;

/// The ServiceLocator's `create_*` seams: the objects that reach outside the process, built from settings the
/// locator hands in. The locator is given its seams, so whatever needs others builds the same production
/// ServiceLocator with seams of its own, which call ServiceLocatorSeams for every seam they keep.
pub trait ServiceLocatorSeamsProtocol: Send + Sync {
    fn create_configuration_source_provider(&self) -> Arc<dyn ConfigurationSourceProviderProtocol>;

    fn create_json_placeholder_http_client(
        &self,
        json_placeholder_settings: &JsonPlaceholderSettings,
    ) -> ApplicationResult<reqwest::Client>;

    fn create_relational_database_pool(
        &self,
        relational_database_settings: &RelationalDatabaseSettings,
    ) -> ApplicationResult<PgPool>;

    fn create_document_database_client(
        &self,
        document_database_settings: &DocumentDatabaseSettings,
    ) -> InMemoryDocumentDatabaseClient;
}
