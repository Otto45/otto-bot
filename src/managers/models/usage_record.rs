use chrono::{DateTime, Utc};

use crate::managers::models::agent_role::AgentRole;
use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;

/// One model response's usage, as the usage ledger keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    pub session_id: String,
    pub lane_name: String,
    pub role: AgentRole,
    pub response_id: String,
    pub model: String,
    pub requested_service_tier: ServiceTier,
    pub served_service_tier: Option<ServiceTier>,
    pub usage: ModelUsage,
    pub recorded_on: DateTime<Utc>,
}
