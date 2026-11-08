use std::fmt;

use crate::managers::models::openai_transport::OpenAiTransport;

#[derive(Clone, PartialEq)]
pub struct OpenAiResponsesSettings {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub timeout_seconds: f64,
    pub idle_timeout_seconds: f64,
    pub transport: OpenAiTransport,
}

// Written out rather than derived, so the API key never reaches a log or an error message, however the settings are
// printed.
impl fmt::Debug for OpenAiResponsesSettings {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenAiResponsesSettings")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .field("model", &self.model)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("idle_timeout_seconds", &self.idle_timeout_seconds)
            .field("transport", &self.transport)
            .finish()
    }
}
