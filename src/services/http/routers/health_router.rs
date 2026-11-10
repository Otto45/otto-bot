use axum::Json;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::services::http::http_service_state::HttpServiceState;
use crate::services::http::wire_models::health_response::HealthResponse;

// Whether the service is up, for whatever watches it: the one route that needs no token.
pub struct HealthRouter;

impl HealthRouter {
    pub fn router() -> OpenApiRouter<HttpServiceState> {
        OpenApiRouter::new().routes(routes!(get_health))
    }
}

#[utoipa::path(
    get,
    path = "/health",
    security(()),
    responses((status = OK, description = "The service is up", body = HealthResponse)),
)]
async fn get_health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}
