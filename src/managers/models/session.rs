use chrono::{DateTime, Utc};

use crate::managers::models::service_tier::ServiceTier;

/// A session: the work one client asks for, request by request, under the options it was started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: String,
    pub service_tier: ServiceTier,
    pub started_on: DateTime<Utc>,
}
