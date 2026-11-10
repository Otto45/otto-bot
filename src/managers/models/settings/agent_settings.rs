use crate::managers::models::service_tier::ServiceTier;

#[derive(Debug, Clone, PartialEq)]
pub struct AgentSettings {
    /// The tier a session runs at when its client doesn't choose one.
    pub default_service_tier: ServiceTier,
    /// The most model turns one run may take before it is stopped.
    pub max_turns: i64,
    /// The longest one of a stack pack's checks may run.
    pub check_timeout_seconds: f64,
    /// How many input tokens the orchestrator's last request may have had before its conversation is compacted, once
    /// it has replied.
    pub orchestrator_compact_tokens: i64,
    /// How many input tokens any lane's last request may have had before its conversation is compacted between turns,
    /// so a long task doesn't reach the model's limit.
    pub lane_compact_tokens: i64,
    /// How long a session with no run and no client following it is kept in memory; it is resumed from the session
    /// store when it is next used.
    pub idle_session_minutes: i64,
}
