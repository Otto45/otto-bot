use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;

/// How a run that finished went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunSummary {
    pub turns: i64,
    pub usage: ModelUsage,
    /// The tier the last response ran at.
    pub service_tier: Option<ServiceTier>,
}
