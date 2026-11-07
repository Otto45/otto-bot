// A config model: what a configuration source read, before it is validated. Every field can be missing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentConfig {
    pub default_service_tier: Option<String>,
}
