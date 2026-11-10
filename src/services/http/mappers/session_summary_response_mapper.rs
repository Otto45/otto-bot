use crate::managers::models::session_summary::SessionSummary;
use crate::services::http::wire_models::session_summary_response::{SessionListResponse, SessionSummaryResponse};

pub struct SessionSummaryResponseMapper;

impl SessionSummaryResponseMapper {
    /// Its workspace is left out: the app it belongs to is in the path.
    pub fn to_session_summary_response(session_summary: SessionSummary) -> SessionSummaryResponse {
        SessionSummaryResponse {
            session_id: session_summary.session.id,
            service_tier: session_summary.session.service_tier.name().to_string(),
            started_on: session_summary.session.started_on,
            last_active_on: session_summary.last_active_on,
            request_count: session_summary.request_count,
            first_request: session_summary.first_request,
        }
    }

    pub fn to_session_list_response(session_summaries: Vec<SessionSummary>) -> SessionListResponse {
        SessionListResponse {
            sessions: session_summaries
                .into_iter()
                .map(SessionSummaryResponseMapper::to_session_summary_response)
                .collect(),
        }
    }
}
