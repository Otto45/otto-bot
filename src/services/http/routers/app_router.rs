use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::started_project::StartedProject;
use crate::services::http::exception_handlers::application_exception_response::ApplicationExceptionResponse;
use crate::services::http::extractors::json_body::JsonBody;
use crate::services::http::extractors::path_parameters::PathParameters;
use crate::services::http::http_service_state::HttpServiceState;
use crate::services::http::mappers::service_tier_request_mapper::ServiceTierRequestMapper;
use crate::services::http::mappers::session_summary_response_mapper::SessionSummaryResponseMapper;
use crate::services::http::mappers::started_app_response_mapper::StartedAppResponseMapper;
use crate::services::http::wire_models::error_response::ErrorResponse;
use crate::services::http::wire_models::service_tier_request::ServiceTierRequest;
use crate::services::http::wire_models::session_created_response::SessionCreatedResponse;
use crate::services::http::wire_models::session_summary_response::SessionListResponse;
use crate::services::http::wire_models::start_app_request::StartAppRequest;
use crate::services::http::wire_models::start_session_request::StartSessionRequest;
use crate::services::http::wire_models::started_app_response::StartedAppResponse;

// Apps, by name, and the sessions started in them. Each route maps the request in, calls one facade operation, and
// maps what it returns out; it names the errors it can answer with, so they are part of the contract (openapi.json).
pub struct AppRouter;

impl AppRouter {
    pub fn router() -> OpenApiRouter<HttpServiceState> {
        OpenApiRouter::new()
            .routes(routes!(start_app))
            .routes(routes!(list_app_sessions, start_app_session))
    }
}

#[utoipa::path(
    post,
    path = "/apps",
    request_body = StartAppRequest,
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = CREATED, description = "The app was started from its stack's template", body = StartedAppResponse),
        (status = BAD_REQUEST, description = "The name can't be an app's, or there is no such stack", body = ErrorResponse),
        (status = CONFLICT, description = "An app of that name already exists", body = ErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "The body does not fit the request model", body = ErrorResponse),
    ),
)]
async fn start_app(
    State(http_service_state): State<HttpServiceState>,
    JsonBody(start_app_request): JsonBody<StartAppRequest>,
) -> Result<(StatusCode, Json<StartedAppResponse>), ApplicationExceptionResponse> {
    let started_project: StartedProject = http_service_state
        .otto_bot_facade
        .start_app(&start_app_request.stack, &start_app_request.name)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(StartedAppResponseMapper::to_started_app_response(started_project)),
    ))
}

#[utoipa::path(
    get,
    path = "/apps/{app_name}/sessions",
    params(("app_name" = String, Path, description = "The app's kebab-case name")),
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = OK, description = "The app's sessions, most recently active first", body = SessionListResponse),
        (status = NOT_FOUND, description = "There is no app of that name", body = ErrorResponse),
    ),
)]
async fn list_app_sessions(
    State(http_service_state): State<HttpServiceState>,
    PathParameters(app_name): PathParameters<String>,
) -> Result<Json<SessionListResponse>, ApplicationExceptionResponse> {
    let session_summaries: Vec<SessionSummary> =
        http_service_state.otto_bot_facade.list_app_sessions(&app_name).await?;
    Ok(Json(SessionSummaryResponseMapper::to_session_list_response(
        session_summaries,
    )))
}

#[utoipa::path(
    post,
    path = "/apps/{app_name}/sessions",
    params(("app_name" = String, Path, description = "The app's kebab-case name")),
    request_body = StartSessionRequest,
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = CREATED, description = "The session was started", body = SessionCreatedResponse),
        (status = NOT_FOUND, description = "There is no app of that name", body = ErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "The body does not fit the request model", body = ErrorResponse),
    ),
)]
async fn start_app_session(
    State(http_service_state): State<HttpServiceState>,
    PathParameters(app_name): PathParameters<String>,
    JsonBody(start_session_request): JsonBody<StartSessionRequest>,
) -> Result<(StatusCode, Json<SessionCreatedResponse>), ApplicationExceptionResponse> {
    let service_tier: Option<ServiceTier> =
        start_session_request
            .service_tier
            .map(|service_tier_request: ServiceTierRequest| {
                ServiceTierRequestMapper::to_service_tier(service_tier_request)
            });
    let session_id: String = http_service_state
        .otto_bot_facade
        .start_app_session(&app_name, service_tier)
        .await?;
    Ok((StatusCode::CREATED, Json(SessionCreatedResponse { session_id })))
}
