use serde::Deserialize;

// The body the API answers an error status with. Only its type and code are kept: its message can quote the request.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ErrorResource {
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ErrorResponseResource {
    pub error: Option<ErrorResource>,
}
