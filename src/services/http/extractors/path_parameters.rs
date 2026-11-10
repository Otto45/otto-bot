use axum::extract::FromRequestParts;

use crate::services::http::extractors::request_rejection::RequestRejection;

/// axum's `Path`, answering a path parameter that doesn't fit with an ErrorResponse.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(RequestRejection))]
pub struct PathParameters<T>(pub T);
