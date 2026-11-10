use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct StartAppRequest {
    /// The app's name, in any form (e.g. "Orders App"); its kebab-case form (orders-app) names it from then on.
    pub name: String,
    /// The stack it is built in: rust, python or typescript.
    pub stack: String,
}
