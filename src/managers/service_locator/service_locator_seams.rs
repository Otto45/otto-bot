use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use rustls_platform_verifier::BuilderVerifierExt;
use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_environment::ConfigurationSourceProviderEnvironment;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connector::ResponsesWebSocketConnector;
use crate::managers::models::settings::openai_responses_settings::OpenAiResponsesSettings;
use crate::managers::models::settings::relational_database_settings::RelationalDatabaseSettings;
use crate::managers::service_locator::service_locator_seams_protocol::ServiceLocatorSeamsProtocol;

/// How long a statement waits for a connection: long enough for a database that sleeps to wake, short enough that
/// one that can't be reached fails the request rather than hanging it.
const RELATIONAL_DATABASE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a request to the OpenAI API waits to connect: a connection that can't be made fails long before the
/// whole response's timeout, and is then told apart from a response that took too long (only the first is retried).
const OPENAI_CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// The production seams: the environment, the network and the database.
pub struct ServiceLocatorSeams;

impl ServiceLocatorSeamsProtocol for ServiceLocatorSeams {
    fn create_configuration_source_provider(&self) -> Arc<dyn ConfigurationSourceProviderProtocol> {
        Arc::new(ConfigurationSourceProviderEnvironment)
    }

    fn create_openai_responses_http_client(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<reqwest::Client> {
        let timeout: Duration =
            ServiceLocatorSeams::to_timeout(openai_responses_settings.timeout_seconds, "timeout_seconds")?;

        reqwest::Client::builder()
            .connect_timeout(OPENAI_CONNECT_TIMEOUT)
            .timeout(timeout)
            .build()
            .map_err(|exception: reqwest::Error| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiRequest,
                    "OpenAI HTTP client could not be built",
                    ContextualData::new(),
                )
                .with_source(exception)
            })
    }

    fn create_openai_responses_web_socket_connector(
        &self,
        openai_responses_settings: &OpenAiResponsesSettings,
    ) -> ApplicationResult<ResponsesWebSocketConnector> {
        // aws-lc-rs, the provider reqwest names for the HTTP transport, and the platform's verifier, which reqwest
        // uses too, so both transports trust the same certificates.
        let tls_configuration: rustls::ClientConfig =
            rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
                .with_safe_default_protocol_versions()
                .and_then(
                    |builder: rustls::ConfigBuilder<rustls::ClientConfig, rustls::WantsVerifier>| {
                        builder.with_platform_verifier()
                    },
                )
                .map_err(|exception: rustls::Error| {
                    ApplicationBaseException::new(
                        OpenAiResponsesGatewayException::OpenAiConnection,
                        "OpenAI WebSocket TLS configuration could not be built",
                        ContextualData::new(),
                    )
                    .with_source(exception)
                })?
                .with_no_client_auth();
        Ok(ResponsesWebSocketConnector::new(
            &openai_responses_settings.base_url,
            openai_responses_settings.api_key.clone(),
            Arc::new(tls_configuration),
            ServiceLocatorSeams::to_timeout(openai_responses_settings.timeout_seconds, "timeout_seconds")?,
            ServiceLocatorSeams::to_timeout(openai_responses_settings.idle_timeout_seconds, "idle_timeout_seconds")?,
        ))
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

        // It keeps no connection open while otto-bot is idle, so a database that sleeps when nothing is connected
        // to it can. A held lock's connection is taken out of the pool, so locks never use up its connections.
        Ok(PgPoolOptions::new()
            .min_connections(0)
            .max_connections(10)
            .idle_timeout(Duration::from_secs(60))
            .acquire_timeout(RELATIONAL_DATABASE_ACQUIRE_TIMEOUT)
            .connect_lazy_with(connect_options))
    }
}

impl ServiceLocatorSeams {
    // The validator keeps a timeout positive and finite; one too long for a Duration is refused here.
    fn to_timeout(seconds: f64, setting_name: &str) -> ApplicationResult<Duration> {
        Duration::try_from_secs_f64(seconds).map_err(|_: std::time::TryFromFloatSecsError| {
            ApplicationBaseException::new(
                ConfigurationProviderException::InvalidConfiguration,
                "OpenAI configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "openai_responses")
                    .with("errors", format!("{setting_name} is too long")),
            )
        })
    }
}
