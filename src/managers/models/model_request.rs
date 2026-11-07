use crate::managers::models::service_tier::ServiceTier;

/// One request to the model, in the domain's terms; the gateway turns it into the API's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelRequest {
    pub user_text: String,
    pub service_tier: ServiceTier,
}
