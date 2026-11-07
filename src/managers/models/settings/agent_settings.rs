use crate::managers::models::service_tier::ServiceTier;

#[derive(Debug, Clone, PartialEq)]
pub struct AgentSettings {
    /// The tier a session runs at when its client doesn't choose one.
    pub default_service_tier: ServiceTier,
    /// The most model turns one run may take before it is stopped.
    pub max_turns: i64,
    /// The longest one of a stack pack's checks may run.
    pub check_timeout_seconds: f64,
}
