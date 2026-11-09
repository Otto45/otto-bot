/// The JsonPlaceholderGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonPlaceholderGatewayException {
    InvalidUserProfileResponse,
    JsonPlaceholderRequest,
    /// JSON Placeholder failed on its side, is limiting requests, or couldn't be reached, through every retry.
    JsonPlaceholderUnavailable,
    UserProfileNotFound,
}

impl JsonPlaceholderGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            JsonPlaceholderGatewayException::InvalidUserProfileResponse => "InvalidUserProfileResponseException",
            JsonPlaceholderGatewayException::JsonPlaceholderRequest => "JsonPlaceholderRequestException",
            JsonPlaceholderGatewayException::JsonPlaceholderUnavailable => "JsonPlaceholderUnavailableException",
            JsonPlaceholderGatewayException::UserProfileNotFound => "UserProfileNotFoundException",
        }
    }
}
