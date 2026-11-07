/// The JsonPlaceholderGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonPlaceholderGatewayException {
    InvalidUserProfileResponse,
    JsonPlaceholderRequest,
    UserProfileNotFound,
}

impl JsonPlaceholderGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            JsonPlaceholderGatewayException::InvalidUserProfileResponse => "InvalidUserProfileResponseException",
            JsonPlaceholderGatewayException::JsonPlaceholderRequest => "JsonPlaceholderRequestException",
            JsonPlaceholderGatewayException::UserProfileNotFound => "UserProfileNotFoundException",
        }
    }
}
