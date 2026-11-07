use crate::managers::models::service_tier::ServiceTier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSettings {
    /// The tier a session runs at when its client doesn't choose one.
    pub default_service_tier: ServiceTier,
}
