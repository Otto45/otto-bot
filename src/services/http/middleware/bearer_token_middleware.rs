use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, WWW_AUTHENTICATE};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use subtle::ConstantTimeEq;

use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::services::http::http_service_state::HttpServiceState;
use crate::services::http::wire_models::error_response::ErrorResponse;

// Refuses a request that doesn't bear the service's token (`Authorization: Bearer <OTTO_BOT_HTTP_API_TOKEN>`), with
// 401 in the ErrorResponse shape. The token is compared in constant time, so how long a refusal takes says nothing of
// it.
pub struct BearerTokenMiddleware;

impl BearerTokenMiddleware {
    pub async fn handle(State(http_service_state): State<HttpServiceState>, request: Request, next: Next) -> Response {
        let bears_token: bool = request
            .headers()
            .get(AUTHORIZATION)
            .and_then(|authorization: &axum::http::HeaderValue| authorization.to_str().ok())
            .and_then(|authorization: &str| authorization.strip_prefix("Bearer "))
            .is_some_and(|token: &str| bool::from(token.as_bytes().ct_eq(http_service_state.api_token.as_bytes())));
        if bears_token {
            return next.run(request).await;
        }

        // A client error: a warning, without the token it sent.
        ApplicationLogger::warning(
            "A request without the service's token was refused",
            ContextualData::new()
                .with("method", request.method().as_str())
                .with("path", request.uri().path()),
            None,
        );
        (
            StatusCode::UNAUTHORIZED,
            [(WWW_AUTHENTICATE, "Bearer")],
            Json(ErrorResponse {
                error: "The request needs the service's bearer token".to_string(),
            }),
        )
            .into_response()
    }
}
