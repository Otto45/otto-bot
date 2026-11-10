use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::services::http::mappers::http_error_mapper::HttpErrorMapper;
use crate::services::http::wire_models::error_response::ErrorResponse;

/// What a route answers when the facade fails: axum's idiom for an exception handler is an error type that knows
/// its own response. A route returns `Result<_, ApplicationExceptionResponse>`, and `?` turns the facade's
/// exception into one.
pub struct ApplicationExceptionResponse(ApplicationBaseException);

impl From<ApplicationBaseException> for ApplicationExceptionResponse {
    fn from(exception: ApplicationBaseException) -> Self {
        ApplicationExceptionResponse(exception)
    }
}

impl IntoResponse for ApplicationExceptionResponse {
    fn into_response(self) -> Response {
        let exception: ApplicationBaseException = self.0;
        let status_code: StatusCode = HttpErrorMapper::to_status_code(&exception);

        // A client's mistake is a warning, easy to find on its own; ERROR is kept for what an engineer must look at.
        if status_code.is_server_error() {
            ApplicationLogger::log_exception(&exception, LogLevel::Error);
        } else {
            ApplicationLogger::log_exception(&exception, LogLevel::Warning);
        }

        let error_response: ErrorResponse = HttpErrorMapper::to_error_response(&exception, status_code);
        (status_code, Json(error_response)).into_response()
    }
}
