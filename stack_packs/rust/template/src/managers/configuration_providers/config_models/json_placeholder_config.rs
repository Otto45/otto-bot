#[derive(Debug, Clone, Default, PartialEq)]
pub struct JsonPlaceholderConfig {
    pub base_url: Option<String>,
    pub timeout_seconds: Option<f64>,
}
