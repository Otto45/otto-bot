use axum::extract::FromRequestParts;

use crate::services::http::extractors::request_rejection::RequestRejection;

/// axum's `Query`, answering a query string that doesn't fit with an ErrorResponse.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(RequestRejection))]
pub struct QueryParameters<T>(pub T);
