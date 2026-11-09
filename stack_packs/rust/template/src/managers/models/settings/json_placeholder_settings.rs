#[derive(Debug, Clone, PartialEq)]
pub struct JsonPlaceholderSettings {
    pub base_url: String,
    pub timeout_seconds: f64,
    /// How many times a request that failed for a reason that may pass is sent again.
    pub max_retries: i64,
}
