use serde::Deserialize;
use utoipa::ToSchema;

// Request models belong to the framework: axum deserializes each request into them and answers a 4xx itself when
// one doesn't fit, so the routes only ever map well-formed requests into domain models. A request model says what
// shape a request has (its fields and their types) and no more: whether the values are acceptable is the domain
// container's decision (here CustomerProcessor's), so that a caller who sends a blank name is answered with the
// domain's own message.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SaveCustomerRequest {
    pub name: String,
    pub email: String,
}
