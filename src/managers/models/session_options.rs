use crate::managers::models::service_tier::ServiceTier;

/// What a client asks of a new session. Whatever it leaves out comes from configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionOptions {
    pub service_tier: Option<ServiceTier>,
}
