use serde::Deserialize;
use utoipa::ToSchema;

use crate::services::http::wire_models::service_tier_request::ServiceTierRequest;

#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
pub struct StartSessionRequest {
    /// The tier the session runs at [default: the server's configured tier].
    #[serde(default)]
    pub service_tier: Option<ServiceTierRequest>,
}
