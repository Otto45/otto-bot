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

impl RunSummary {
    /// This run and a later one, as one: their turns and usage added, and the later one's tier.
    pub fn plus(self, later: RunSummary) -> RunSummary {
        RunSummary {
            turns: self.turns + later.turns,
            usage: self.usage.plus(later.usage),
            service_tier: later.service_tier.or(self.service_tier),
        }
    }
}
