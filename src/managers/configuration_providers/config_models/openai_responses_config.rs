use std::fmt;

#[derive(Clone, Default, PartialEq)]
pub struct OpenAiResponsesConfig {
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<f64>,
    pub idle_timeout_seconds: Option<f64>,
    pub transport: Option<String>,
}

// Written out rather than derived, so the API key never reaches a log or an error message.
impl fmt::Debug for OpenAiResponsesConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenAiResponsesConfig")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_: &String| "[REDACTED]"))
            .field("model", &self.model)
            .field("timeout_seconds", &self.timeout_seconds)
            .field("idle_timeout_seconds", &self.idle_timeout_seconds)
            .field("transport", &self.transport)
            .finish()
    }
}
