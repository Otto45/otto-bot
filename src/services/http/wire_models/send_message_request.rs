use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SendMessageRequest {
    /// What the person wrote.
    pub text: String,
}
