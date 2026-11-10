use std::path::PathBuf;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session::Session;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::session_request::SessionRequest;
use crate::managers::models::session_summary::SessionSummary;
use crate::managers::models::user_request::UserRequest;

// Decides what a session is from what its client asked for, and whether a request can be run.
pub struct SessionProcessor {
    default_service_tier: ServiceTier,
}

impl SessionProcessor {
    pub fn new(default_service_tier: ServiceTier) -> Self {
        SessionProcessor { default_service_tier }
    }

    /// A new session: what the client chose, and the configured defaults for what it left out. The workspace is the
    /// client's, resolved to its canonical path.
    pub fn create_session(
        &self,
        session_options: &SessionOptions,
        workspace_directory: PathBuf,
        started_on: DateTime<Utc>,
    ) -> Session {
        Session {
            id: Uuid::new_v4().simple().to_string(),
            workspace_directory,
            service_tier: session_options.service_tier.unwrap_or(self.default_service_tier),
            started_on,
        }
    }

    /// The session as a client lists it, from the requests it was given (oldest first).
    pub fn summarize_session(&self, session: Session, session_requests: &[SessionRequest]) -> SessionSummary {
        SessionSummary {
            first_request: session_requests
                .first()
                .map(|session_request: &SessionRequest| session_request.text.clone()),
            request_count: session_requests.len(),
            last_active_on: session_requests
                .last()
                .map_or(session.started_on, |session_request: &SessionRequest| {
                    session_request.submitted_on
                }),
            session,
        }
    }

    /// Most recently active first, so the session last worked on heads the list.
    pub fn sort_session_summaries(&self, session_summaries: &mut [SessionSummary]) {
        session_summaries.sort_by(|left: &SessionSummary, right: &SessionSummary| {
            right
                .last_active_on
                .cmp(&left.last_active_on)
                .then_with(|| left.session.id.cmp(&right.session.id))
        });
    }

    pub fn check_user_request(&self, user_request: &UserRequest) -> ApplicationResult<()> {
        if user_request.text.trim().is_empty() {
            return Err(ApplicationBaseException::new(
                ExceptionKind::InvalidUserRequest,
                "The request is empty",
                ContextualData::new(),
            ));
        }
        Ok(())
    }
}
