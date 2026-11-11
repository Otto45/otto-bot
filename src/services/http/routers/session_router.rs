use std::convert::Infallible;
use std::pin::Pin;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use futures_util::{Stream, StreamExt};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::managers::models::session_event_record::SessionEventRecord;
use crate::managers::models::session_event_replay::SessionEventReplay;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::transcript_entry::TranscriptEntry;
use crate::managers::models::user_request::UserRequest;
use crate::services::http::exception_handlers::application_exception_response::ApplicationExceptionResponse;
use crate::services::http::extractors::json_body::JsonBody;
use crate::services::http::extractors::path_parameters::PathParameters;
use crate::services::http::extractors::query_parameters::QueryParameters;
use crate::services::http::http_service_state::HttpServiceState;
use crate::services::http::mappers::session_event_replay_mapper::SessionEventReplayMapper;
use crate::services::http::mappers::session_event_response_mapper::SessionEventResponseMapper;
use crate::services::http::mappers::session_summary_response_mapper::SessionSummaryResponseMapper;
use crate::services::http::mappers::transcript_response_mapper::TranscriptResponseMapper;
use crate::services::http::wire_models::error_response::ErrorResponse;
use crate::services::http::wire_models::event_query::EventQuery;
use crate::services::http::wire_models::send_message_request::SendMessageRequest;
use crate::services::http::wire_models::session_event_response::SessionEventResponse;
use crate::services::http::wire_models::session_summary_response::SessionSummaryResponse;
use crate::services::http::wire_models::transcript_response::TranscriptResponse;

const LAST_EVENT_ID: &str = "last-event-id";

// What a session's event stream is made of, boxed so its type can be written out.
type EventStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

// One session of an app. Every route resumes the session first if the service isn't serving it (after a restart, or
// once it was let go for being idle), so a client never has to.
pub struct SessionRouter;

impl SessionRouter {
    pub fn router() -> OpenApiRouter<HttpServiceState> {
        OpenApiRouter::new()
            .routes(routes!(get_session))
            .routes(routes!(send_message, get_messages))
            .routes(routes!(cancel_run))
            .routes(routes!(follow_events))
    }
}

#[utoipa::path(
    get,
    path = "/apps/{app_name}/sessions/{session_id}",
    params(
        ("app_name" = String, Path, description = "The app's kebab-case name"),
        ("session_id" = String, Path),
    ),
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = OK, description = "The session", body = SessionSummaryResponse),
        (status = NOT_FOUND, description = "There is no app of that name, or no session of it with that id", body = ErrorResponse),
    ),
)]
async fn get_session(
    State(http_service_state): State<HttpServiceState>,
    PathParameters((app_name, session_id)): PathParameters<(String, String)>,
) -> Result<Json<SessionSummaryResponse>, ApplicationExceptionResponse> {
    let session_summary: SessionSummary = http_service_state
        .otto_bot_facade
        .resume_app_session(&app_name, &session_id)
        .await?;
    Ok(Json(SessionSummaryResponseMapper::to_session_summary_response(
        session_summary,
    )))
}

#[utoipa::path(
    post,
    path = "/apps/{app_name}/sessions/{session_id}/messages",
    params(
        ("app_name" = String, Path, description = "The app's kebab-case name"),
        ("session_id" = String, Path),
    ),
    request_body = SendMessageRequest,
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = ACCEPTED, description = "otto is working on it: follow the session's events"),
        (status = BAD_REQUEST, description = "The message is empty", body = ErrorResponse),
        (status = NOT_FOUND, description = "There is no app of that name, or no session of it with that id", body = ErrorResponse),
        (status = CONFLICT, description = "The session, or another session of the app, is still working on a request", body = ErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "The body does not fit the request model", body = ErrorResponse),
    ),
)]
async fn send_message(
    State(http_service_state): State<HttpServiceState>,
    PathParameters((app_name, session_id)): PathParameters<(String, String)>,
    JsonBody(send_message_request): JsonBody<SendMessageRequest>,
) -> Result<StatusCode, ApplicationExceptionResponse> {
    http_service_state
        .otto_bot_facade
        .resume_app_session(&app_name, &session_id)
        .await?;
    http_service_state
        .otto_bot_facade
        .submit(
            &session_id,
            &UserRequest {
                text: send_message_request.text,
            },
        )
        .await?;
    Ok(StatusCode::ACCEPTED)
}

#[utoipa::path(
    get,
    path = "/apps/{app_name}/sessions/{session_id}/messages",
    params(
        ("app_name" = String, Path, description = "The app's kebab-case name"),
        ("session_id" = String, Path),
    ),
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = OK, description = "What the person and otto said, oldest first", body = TranscriptResponse),
        (status = NOT_FOUND, description = "There is no app of that name, or no session of it with that id", body = ErrorResponse),
    ),
)]
async fn get_messages(
    State(http_service_state): State<HttpServiceState>,
    PathParameters((app_name, session_id)): PathParameters<(String, String)>,
) -> Result<Json<TranscriptResponse>, ApplicationExceptionResponse> {
    let transcript_entries: Vec<TranscriptEntry> = http_service_state
        .otto_bot_facade
        .get_app_session_transcript(&app_name, &session_id)
        .await?;
    Ok(Json(TranscriptResponseMapper::to_transcript_response(
        transcript_entries,
    )))
}

#[utoipa::path(
    post,
    path = "/apps/{app_name}/sessions/{session_id}/cancel",
    params(
        ("app_name" = String, Path, description = "The app's kebab-case name"),
        ("session_id" = String, Path),
    ),
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = NO_CONTENT, description = "The run under way, if any, is stopping: its end comes as run_cancelled"),
        (status = NOT_FOUND, description = "There is no app of that name, or no session of it with that id", body = ErrorResponse),
    ),
)]
async fn cancel_run(
    State(http_service_state): State<HttpServiceState>,
    PathParameters((app_name, session_id)): PathParameters<(String, String)>,
) -> Result<StatusCode, ApplicationExceptionResponse> {
    http_service_state
        .otto_bot_facade
        .resume_app_session(&app_name, &session_id)
        .await?;
    http_service_state.otto_bot_facade.cancel(&session_id)?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/apps/{app_name}/sessions/{session_id}/events",
    params(
        ("app_name" = String, Path, description = "The app's kebab-case name"),
        ("session_id" = String, Path),
        EventQuery,
        ("Last-Event-ID" = Option<String>, Header, description = "The id of the last event seen, as an EventSource sends it when it reconnects"),
    ),
    responses(
        (status = UNAUTHORIZED, description = "The request doesn't bear the service's token", body = ErrorResponse),
        (status = OK, description = "The session's events as server-sent events, from the current run's start (or after the event named), for as long as the client stays", content_type = "text/event-stream", body = SessionEventResponse),
        (status = NOT_FOUND, description = "There is no app of that name, or no session of it with that id", body = ErrorResponse),
    ),
)]
async fn follow_events(
    State(http_service_state): State<HttpServiceState>,
    PathParameters((app_name, session_id)): PathParameters<(String, String)>,
    QueryParameters(event_query): QueryParameters<EventQuery>,
    headers: HeaderMap,
) -> Result<Sse<KeepAliveStream<EventStream<Result<Event, Infallible>>>>, ApplicationExceptionResponse> {
    http_service_state
        .otto_bot_facade
        .resume_app_session(&app_name, &session_id)
        .await?;
    let session_event_replay: SessionEventReplay = SessionEventReplayMapper::to_session_event_replay(
        event_query.after,
        headers
            .get(LAST_EVENT_ID)
            .and_then(|last_event_id: &axum::http::HeaderValue| last_event_id.to_str().ok()),
    );
    let (missed, receiver): (Vec<SessionEventRecord>, broadcast::Receiver<SessionEventRecord>) = http_service_state
        .otto_bot_facade
        .subscribe(&session_id, session_event_replay)?;

    // A client too far behind is cut off: it reconnects with the last id it saw, and is sent what it missed.
    let live_events: EventStream<SessionEventRecord> = Box::pin(
        BroadcastStream::new(receiver)
            .take_while(|received: &Result<SessionEventRecord, BroadcastStreamRecvError>| {
                std::future::ready(received.is_ok())
            })
            .filter_map(|received: Result<SessionEventRecord, BroadcastStreamRecvError>| {
                std::future::ready(received.ok())
            }),
    );
    let mut shutting_down: tokio::sync::watch::Receiver<bool> = http_service_state.shutting_down.clone();
    let events: EventStream<Result<Event, Infallible>> = Box::pin(
        futures_util::stream::iter(missed)
            .chain(live_events)
            .map(|session_event_record: SessionEventRecord| {
                Ok::<Event, Infallible>(SessionEventResponseMapper::to_server_sent_event(session_event_record))
            })
            // Shutting down ends the stream, so the server can finish.
            .take_until(async move {
                shutting_down.wait_for(|stopping: &bool| *stopping).await.ok();
            }),
    );
    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}
