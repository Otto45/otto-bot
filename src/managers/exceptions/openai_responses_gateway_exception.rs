/// The OpenAiResponsesGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesGatewayException {
    /// The request could not be sent, or the API answered with an error status.
    OpenAiRequest,
    /// The API answered with a body the harness cannot read.
    InvalidOpenAiResponse,
    /// The API answered, but the response did not complete (e.g. it ran out of output tokens).
    IncompleteOpenAiResponse,
}

impl OpenAiResponsesGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            OpenAiResponsesGatewayException::OpenAiRequest => "OpenAiRequestException",
            OpenAiResponsesGatewayException::InvalidOpenAiResponse => "InvalidOpenAiResponseException",
            OpenAiResponsesGatewayException::IncompleteOpenAiResponse => "IncompleteOpenAiResponseException",
        }
    }
}
