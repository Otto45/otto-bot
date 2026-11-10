use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::services::http::wire_models::error_response::ErrorResponse;

/// How axum's own answer to a request that doesn't fit (a body that is not JSON or not the request model, a path
/// parameter of the wrong type) goes over the wire: axum's status and message, in the ErrorResponse every other
/// error is answered with, so that callers handle one shape of error.
pub struct RequestRejection {
    status_code: StatusCode,
    error: String,
}

impl From<JsonRejection> for RequestRejection {
    fn from(rejection: JsonRejection) -> Self {
        RequestRejection {
            status_code: rejection.status(),
            error: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for RequestRejection {
    fn from(rejection: PathRejection) -> Self {
        RequestRejection {
            status_code: rejection.status(),
            error: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for RequestRejection {
    fn from(rejection: QueryRejection) -> Self {
        RequestRejection {
            status_code: rejection.status(),
            error: rejection.body_text(),
        }
    }
}

impl IntoResponse for RequestRejection {
    fn into_response(self) -> Response {
        (self.status_code, Json(ErrorResponse { error: self.error })).into_response()
    }
}
