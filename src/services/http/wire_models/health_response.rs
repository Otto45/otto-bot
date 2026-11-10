use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    /// ok.
    pub status: String,
}
