use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::service_tier::ServiceTier;

pub struct ValidatorAgent;

impl ValidatorAgent {
    pub fn validate_agent_config(config: &AgentConfig) -> ApplicationResult<()> {
        if let Some(error) = ValidatorString::validate("default_service_tier", config.default_service_tier.as_deref()) {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::MissingConfiguration,
                "Agent configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "agent")
                    .with("errors", error),
            ));
        }

        if let Some(default_service_tier) = config.default_service_tier.as_deref()
            && ServiceTier::from_name(default_service_tier).is_none()
        {
            return Err(ApplicationBaseException::new(
                ConfigurationProviderException::InvalidConfiguration,
                "Agent configuration is invalid",
                ContextualData::new()
                    .with("setting_name", "agent")
                    .with("errors", "default_service_tier is not one of standard, fast, ultrafast"),
            ));
        }
        Ok(())
    }
}
