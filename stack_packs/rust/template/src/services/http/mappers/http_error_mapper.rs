use axum::http::StatusCode;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;
use crate::services::http::wire_models::error_response::ErrorResponse;

// Translates the domain container's exceptions into HTTP errors. Malformed requests never get here: axum answers
// those itself (see RequestRejection).
pub struct HttpErrorMapper;

impl HttpErrorMapper {
    pub fn to_status_code(exception: &ApplicationBaseException) -> StatusCode {
        // Most specific first: the not-found exceptions are also gateway exceptions. Every family is named, so a new
        // one does not compile until it is given a status here.
        match exception.kind() {
            ExceptionKind::InvalidCustomer => StatusCode::BAD_REQUEST,
            ExceptionKind::RelationalDatabaseGateway(RelationalDatabaseGatewayException::CustomerNotFound)
            | ExceptionKind::JsonPlaceholderGateway(JsonPlaceholderGatewayException::UserProfileNotFound) => {
                StatusCode::NOT_FOUND
            }
            // An upstream service down for now, through every retry: the caller can try again later.
            ExceptionKind::JsonPlaceholderGateway(JsonPlaceholderGatewayException::JsonPlaceholderUnavailable) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            ExceptionKind::JsonPlaceholderGateway(_) => StatusCode::BAD_GATEWAY,
            ExceptionKind::ConfigurationProvider(_)
            | ExceptionKind::RelationalDatabaseGateway(_)
            | ExceptionKind::DocumentDatabaseGateway(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_error_response(exception: &ApplicationBaseException, status_code: StatusCode) -> ErrorResponse {
        // A client error answers with the exception's own message: those exceptions are the ones listed above as
        // 4xx, and their messages are written for the person using the app. Every other message and every context
        // describes internals (tables, SQL statements, upstream services) that callers must not see; they are
        // logged instead.
        if !status_code.is_server_error() {
            return ErrorResponse {
                error: exception.message().to_string(),
            };
        }
        if status_code == StatusCode::SERVICE_UNAVAILABLE {
            return ErrorResponse {
                error: "An upstream service is unavailable for now; try again later".to_string(),
            };
        }
        if status_code == StatusCode::BAD_GATEWAY {
            return ErrorResponse {
                error: "An upstream service failed".to_string(),
            };
        }
        ErrorResponse {
            error: "The request could not be completed".to_string(),
        }
    }
}
