use chrono::{DateTime, Utc};

/// A request the session was given, kept in the order it came, so a session can be told apart from another and its
/// last activity known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRequest {
    pub text: String,
    pub submitted_on: DateTime<Utc>,
}
