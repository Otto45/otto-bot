use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::models::service_tier::ServiceTier;
use crate::managers::models::session::Session;
use crate::managers::models::session_options::SessionOptions;
use crate::managers::models::user_request::UserRequest;

// Decides what a session is from what its client asked for, and whether a request can be run.
pub struct SessionProcessor {
    default_service_tier: ServiceTier,
}

impl SessionProcessor {
    pub fn new(default_service_tier: ServiceTier) -> Self {
        SessionProcessor { default_service_tier }
    }

    /// A new session: what the client chose, and the configured defaults for what it left out.
    pub fn create_session(&self, session_options: &SessionOptions, started_on: DateTime<Utc>) -> Session {
        Session {
            id: Uuid::new_v4().simple().to_string(),
            service_tier: session_options.service_tier.unwrap_or(self.default_service_tier),
            started_on,
        }
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
