use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::configuration_providers::config_models::openai_responses_config::OpenAiResponsesConfig;
use crate::managers::exceptions::application_base_exception::ApplicationResult;

pub trait ConfigurationSourceProviderProtocol: Send + Sync {
    fn get_agent_config(&self) -> ApplicationResult<AgentConfig>;

    fn get_openai_responses_config(&self) -> ApplicationResult<OpenAiResponsesConfig>;
}
