use crate::managers::models::model_usage::ModelUsage;
use crate::managers::models::service_tier::ServiceTier;

/// What the model answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelResponse {
    pub id: String,
    pub text: String,
    pub usage: ModelUsage,
    /// The tier the response actually ran at: a fast request can be served at the standard tier under load. None
    /// when the API names a tier the harness never asks for.
    pub service_tier: Option<ServiceTier>,
}
