use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserProfileResponse {
    pub id: i64,
    pub name: String,
    pub username: String,
    pub email: String,
    pub city: String,
    pub company_name: String,
}
