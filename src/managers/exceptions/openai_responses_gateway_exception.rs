/// The OpenAiResponsesGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesGatewayException {
    /// The request could not be sent, or the API answered with an error status.
    OpenAiRequest,
    /// The API answered with a body the harness cannot read.
    InvalidOpenAiResponse,
    /// The API answered, but the response did not complete (e.g. it ran out of output tokens).
    IncompleteOpenAiResponse,
    /// A WebSocket connection could not be opened, or was lost before the response came.
    OpenAiConnection,
    /// The response a request continues is no longer held by the connection (evicted, or the connection is new).
    PreviousResponseNotFound,
}

impl OpenAiResponsesGatewayException {
    pub fn name(self) -> &'static str {
        match self {
            OpenAiResponsesGatewayException::OpenAiRequest => "OpenAiRequestException",
            OpenAiResponsesGatewayException::InvalidOpenAiResponse => "InvalidOpenAiResponseException",
            OpenAiResponsesGatewayException::IncompleteOpenAiResponse => "IncompleteOpenAiResponseException",
            OpenAiResponsesGatewayException::OpenAiConnection => "OpenAiConnectionException",
            OpenAiResponsesGatewayException::PreviousResponseNotFound => "PreviousResponseNotFoundException",
        }
    }
}
