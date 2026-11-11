use axum::Json;
use axum::http::StatusCode;

use crate::services::http::wire_models::error_response::ErrorResponse;

/// What axum answers a request no route takes (a path that is no route's, or a method its route doesn't accept), in
/// the ErrorResponse every other error is answered with, so that callers handle one shape of error; axum's own answer
/// has no body. A 405 keeps the `Allow` header axum sets.
pub struct UnmatchedRequestHandler;

impl UnmatchedRequestHandler {
    pub async fn handle_not_found() -> (StatusCode, Json<ErrorResponse>) {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Not Found".to_string(),
            }),
        )
    }

    pub async fn handle_method_not_allowed() -> (StatusCode, Json<ErrorResponse>) {
        (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(ErrorResponse {
                error: "Method Not Allowed".to_string(),
            }),
        )
    }
}
