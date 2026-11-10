use axum::http::StatusCode;

use crate::managers::exceptions::application_base_exception::ApplicationBaseException;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::exceptions::otto_bot_manager_exception::OttoBotManagerException;
use crate::services::http::wire_models::error_response::ErrorResponse;

// Translates the domain container's exceptions into HTTP errors. Malformed requests never get here: axum answers
// those itself (see RequestRejection).
pub struct HttpErrorMapper;

impl HttpErrorMapper {
    pub fn to_status_code(exception: &ApplicationBaseException) -> StatusCode {
        // Every family is named, so a new one does not compile until it is given a status here.
        match exception.kind() {
            ExceptionKind::OttoBotManager(
                OttoBotManagerException::SessionNotFound | OttoBotManagerException::AppNotFound,
            ) => StatusCode::NOT_FOUND,
            ExceptionKind::OttoBotManager(
                OttoBotManagerException::SessionBusy
                | OttoBotManagerException::AppBusy
                | OttoBotManagerException::AppAlreadyExists,
            ) => StatusCode::CONFLICT,
            // An empty message, an app's name that can't be one, a stack there is no pack (or template) for.
            ExceptionKind::InvalidUserRequest | ExceptionKind::ProjectBootstrapEngine(_) => StatusCode::BAD_REQUEST,
            // Apps are addressed by name, so a workspace's own errors (which name its path) are the server's.
            ExceptionKind::WorkspaceGateway(_)
            | ExceptionKind::ConfigurationProvider(_)
            | ExceptionKind::AgentRunEngine(_)
            | ExceptionKind::CommandRunnerGateway(_)
            | ExceptionKind::OpenAiResponsesGateway(_)
            | ExceptionKind::SessionStoreGateway(_)
            | ExceptionKind::StackPackStoreGateway(_)
            | ExceptionKind::InvalidStackPack => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_error_response(exception: &ApplicationBaseException, status_code: StatusCode) -> ErrorResponse {
        // A client error answers with the exception's own message: those exceptions are the ones listed above as
        // 4xx, and their messages are written for the person using the app. Every other message and every context
        // describes internals (paths, the model's API) that callers must not see; they are logged instead.
        if !status_code.is_server_error() {
            return ErrorResponse {
                error: exception.message().to_string(),
            };
        }
        ErrorResponse {
            error: "The request could not be completed".to_string(),
        }
    }
}
