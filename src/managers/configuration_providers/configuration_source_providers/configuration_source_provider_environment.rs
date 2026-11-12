use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::configuration_providers::config_models::storage_config::StorageConfig;
use crate::managers::configuration_providers::configuration_source_providers::configuration_source_provider_protocol::ConfigurationSourceProviderProtocol;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;

// The stack packs this build of otto-bot came with, unless OTTO_BOT_STACK_PACKS_DIRECTORY says otherwise.
const DEFAULT_STACK_PACKS_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/stack_packs");

pub struct ConfigurationSourceProviderEnvironment;

impl ConfigurationSourceProviderProtocol for ConfigurationSourceProviderEnvironment {
    fn get_agent_config(&self) -> ApplicationResult<AgentConfig> {
        Ok(AgentConfig {
            default_service_tier: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_SERVICE_TIER",
                "standard",
            )),
            max_turns: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_MAX_TURNS",
                "60",
            )?),
            check_timeout_seconds: Some(ConfigurationSourceProviderEnvironment::get_float(
                "OTTO_BOT_CHECK_TIMEOUT_SECONDS",
                "600",
            )?),
            orchestrator_compact_tokens: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_ORCHESTRATOR_COMPACT_TOKENS",
                "400000",
            )?),
            idle_session_minutes: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_IDLE_SESSION_MINUTES",
                "30",
            )?),
            // A run holds its app's and its session's lock: in Postgres, two connections of their own.
            max_concurrent_runs: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_MAX_CONCURRENT_RUNS",
                "8",
            )?),
            max_live_sessions: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_MAX_LIVE_SESSIONS",
                "200",
            )?),
            max_event_streams: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_MAX_EVENT_STREAMS",
                "256",
            )?),
            // gpt-6.1-sol takes up to 922,000 input tokens: this leaves room for one more turn of large reads.
            lane_compact_tokens: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_LANE_COMPACT_TOKENS",
                "600000",
            )?),
        })
    }

    fn get_storage_config(&self) -> ApplicationResult<StorageConfig> {
        // Under the user's home folder unless OTTO_BOT_HOME says otherwise; with neither, the validator says so.
        let home_directory: Option<String> = std::env::var("OTTO_BOT_HOME").ok().or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|user_home_directory: String| format!("{user_home_directory}/.otto-bot"))
        });
        Ok(StorageConfig {
            home_directory,
            stack_packs_directory: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_STACK_PACKS_DIRECTORY",
                DEFAULT_STACK_PACKS_DIRECTORY,
            )),
            workspaces_directory: std::env::var("OTTO_BOT_WORKSPACES_DIRECTORY").ok(),
            relational_database_connection_string: std::env::var("OTTO_BOT_RELATIONAL_DATABASE_CONNECTION_STRING").ok(),
        })
    }

    fn get_openai_responses_config(&self) -> ApplicationResult<OpenAiResponsesConfig> {
        Ok(OpenAiResponsesConfig {
            base_url: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_OPENAI_BASE_URL",
                "https://api.openai.com/v1",
            )),
            // The name every OpenAI client reads, so the sandbox VM sets one variable for all of them. It has no
            // default: without it, creating the facade fails.
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            model: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_MODEL",
                "gpt-6.1-sol",
            )),
            // Long: a response that reasons at high effort can take minutes.
            timeout_seconds: Some(ConfigurationSourceProviderEnvironment::get_float(
                "OTTO_BOT_OPENAI_TIMEOUT_SECONDS",
                "600",
            )?),
            // A response streams events as it goes (the longest silence measured, while reasoning at max effort, was
            // under 8 seconds), so one silent this long has stalled.
            idle_timeout_seconds: Some(ConfigurationSourceProviderEnvironment::get_float(
                "OTTO_BOT_OPENAI_IDLE_TIMEOUT_SECONDS",
                "120",
            )?),
            // Enough to ride out a brief outage on the API's side (the waits add up to about 7 seconds) without
            // holding a run for long when it lasts.
            max_retries: Some(ConfigurationSourceProviderEnvironment::get_integer(
                "OTTO_BOT_OPENAI_MAX_RETRIES",
                "3",
            )?),
            transport: Some(ConfigurationSourceProviderEnvironment::get_string(
                "OTTO_BOT_OPENAI_TRANSPORT",
                "websocket",
            )),
        })
    }
}

impl ConfigurationSourceProviderEnvironment {
    fn get_string(environment_variable_name: &str, default_value: &str) -> String {
        std::env::var(environment_variable_name).unwrap_or_else(|_: std::env::VarError| default_value.to_string())
    }

    fn get_integer(environment_variable_name: &str, default_value: &str) -> ApplicationResult<i64> {
        let value: String =
            ConfigurationSourceProviderEnvironment::get_string(environment_variable_name, default_value);
        value
            .trim()
            .parse::<i64>()
            .map_err(|exception: std::num::ParseIntError| {
                ApplicationBaseException::new(
                    ConfigurationProviderException::InvalidConfiguration,
                    "Environment variable is not a whole number",
                    ContextualData::new().with("environment_variable_name", environment_variable_name),
                )
                .with_source(exception)
            })
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
