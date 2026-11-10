// A config model: what a configuration source read, before it is validated. Every field can be missing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AgentConfig {
    pub default_service_tier: Option<String>,
    pub max_turns: Option<i64>,
    pub check_timeout_seconds: Option<f64>,
    pub orchestrator_compact_tokens: Option<i64>,
    pub lane_compact_tokens: Option<i64>,
}
