use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::configuration_providers::config_models::agent_config::AgentConfig;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::configuration_provider_exception::ConfigurationProviderException;
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::models::service_tier::ServiceTier;

pub struct ValidatorAgent;

impl ValidatorAgent {
    pub fn validate_agent_config(config: &AgentConfig) -> ApplicationResult<()> {
        let mut missing: Vec<String> = Vec::new();
        if let Some(error) = ValidatorString::validate("default_service_tier", config.default_service_tier.as_deref()) {
            missing.push(error);
        }
        if config.max_turns.is_none() {
            missing.push("max_turns is missing".to_string());
        }
        if config.check_timeout_seconds.is_none() {
            missing.push("check_timeout_seconds is missing".to_string());
        }
        if config.orchestrator_compact_tokens.is_none() {
            missing.push("orchestrator_compact_tokens is missing".to_string());
        }
        if config.lane_compact_tokens.is_none() {
            missing.push("lane_compact_tokens is missing".to_string());
        }
        if config.idle_session_minutes.is_none() {
            missing.push("idle_session_minutes is missing".to_string());
        }
        if !missing.is_empty() {
            return Err(ValidatorAgent::to_exception(
                ConfigurationProviderException::MissingConfiguration,
                missing,
            ));
        }

        let mut invalid: Vec<String> = Vec::new();
        if let Some(default_service_tier) = config.default_service_tier.as_deref()
            && ServiceTier::from_name(default_service_tier).is_none()
        {
            invalid.push("default_service_tier is not one of standard, fast, ultrafast".to_string());
        }
        if let Some(max_turns) = config.max_turns
            && max_turns <= 0
        {
            invalid.push("max_turns is not positive".to_string());
        }
        for (setting_name, tokens) in [
            ("orchestrator_compact_tokens", config.orchestrator_compact_tokens),
            ("lane_compact_tokens", config.lane_compact_tokens),
            ("idle_session_minutes", config.idle_session_minutes),
        ] {
            if let Some(tokens) = tokens
                && tokens <= 0
            {
                invalid.push(format!("{setting_name} is not positive"));
            }
        }
        if let Some(check_timeout_seconds) = config.check_timeout_seconds
            && !(check_timeout_seconds.is_finite() && check_timeout_seconds > 0.0)
        {
            invalid.push("check_timeout_seconds is not a positive, finite number".to_string());
        }
        if !invalid.is_empty() {
            return Err(ValidatorAgent::to_exception(
                ConfigurationProviderException::InvalidConfiguration,
                invalid,
            ));
        }
        Ok(())
    }

    fn to_exception(kind: ConfigurationProviderException, errors: Vec<String>) -> ApplicationBaseException {
        ApplicationBaseException::new(
            kind,
            "Agent configuration is invalid",
            ContextualData::new()
                .with("setting_name", "agent")
                .with("errors", errors.join("; ")),
        )
    }
}
