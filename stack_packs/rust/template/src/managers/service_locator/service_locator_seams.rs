use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_environment::ConfigurationSourceProviderEnvironment;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::data_layer::managers::gateways::document_database::clients::in_memory_document_database_client::InMemoryDocumentDatabaseClient;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::models::settings::document_database_settings::DocumentDatabaseSettings;
use crate::managers::models::settings::json_placeholder_settings::JsonPlaceholderSettings;
use crate::managers::models::settings::relational_database_settings::RelationalDatabaseSettings;
use crate::managers::service_locator::service_locator_seams_protocol::ServiceLocatorSeamsProtocol;

/// How long a statement waits for a connection: long enough for a database that sleeps to wake, short enough that
/// one that can't be reached fails the request rather than hanging it.
const RELATIONAL_DATABASE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);

/// The production seams: the environment, the network and the database drivers.
pub struct ServiceLocatorSeams;

impl ServiceLocatorSeamsProtocol for ServiceLocatorSeams {
    fn create_configuration_source_provider(&self) -> Arc<dyn ConfigurationSourceProviderProtocol> {
        Arc::new(ConfigurationSourceProviderEnvironment)
    }

    fn create_json_placeholder_http_client(
        &self,
        json_placeholder_settings: &JsonPlaceholderSettings,
    ) -> ApplicationResult<reqwest::Client> {
        // The validator keeps the timeout positive and finite; one too long for a Duration is refused here.
        let timeout: Duration = Duration::try_from_secs_f64(json_placeholder_settings.timeout_seconds).map_err(
            |_: std::time::TryFromFloatSecsError| {
                ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "JSON Placeholder configuration is invalid",
                    ContextualData::new()
                        .with("setting_name", "json_placeholder")
                        .with("errors", "timeout_seconds is too long"),
                )
            },
        )?;

        reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|exception: reqwest::Error| {
                ApplicationBaseException::new(
                    JsonPlaceholderGatewayException::JsonPlaceholderRequest,
                    "JSON Placeholder HTTP client could not be built",
                    ContextualData::new(),
                )
                .with_source(exception)
            })
    }

    fn create_relational_database_pool(
        &self,
        relational_database_settings: &RelationalDatabaseSettings,
    ) -> ApplicationResult<PgPool> {
        // Only parsed here: nothing connects until the gateway first sends a statement. The parser's own error is
        // not kept, because it can quote the connection string's password.
        let connect_options: PgConnectOptions =
            PgConnectOptions::from_str(&relational_database_settings.connection_string).map_err(|_: sqlx::Error| {
                ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "Relational database configuration is invalid",
                    ContextualData::new()
                        .with("setting_name", "relational_database")
                        .with("errors", "connection_string is not a Postgres connection string"),
                )
            })?;

        // It keeps no connection open while the app is idle, so a database that sleeps when nothing is connected
        // to it can.
        Ok(PgPoolOptions::new()
            .min_connections(0)
            .max_connections(10)
            .idle_timeout(Duration::from_secs(60))
            .acquire_timeout(RELATIONAL_DATABASE_ACQUIRE_TIMEOUT)
            .connect_lazy_with(connect_options))
    }

    fn create_document_database_client(
        &self,
        document_database_settings: &DocumentDatabaseSettings,
    ) -> InMemoryDocumentDatabaseClient {
        InMemoryDocumentDatabaseClient::new(&document_database_settings.connection_string)
    }
}
