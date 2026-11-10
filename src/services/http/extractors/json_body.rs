use axum::extract::FromRequest;

use crate::services::http::extractors::request_rejection::RequestRejection;

/// axum's `Json`, answering a body that doesn't fit with an ErrorResponse.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(RequestRejection))]
pub struct JsonBody<T>(pub T);
