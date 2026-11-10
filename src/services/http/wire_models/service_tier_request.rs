use serde::Deserialize;
use utoipa::ToSchema;

/// How fast, and at what price, the model serves the session's requests.
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTierRequest {
    Standard,
    Fast,
    Ultrafast,
}
