/// The OpenAiResponsesGateway's exceptions: one family of ExceptionKind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiResponsesGatewayException {
    /// The API refused the request, for a reason sending it again won't change (it is malformed, unauthorized, over
    /// a quota).
    OpenAiRequest,
    /// The API failed on its side, or is limiting requests, or couldn't be reached: sending the request again, a little
    /// later, may well succeed, so the gateway does, a few times.
    OpenAiUnavailable,
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
            OpenAiResponsesGatewayException::OpenAiUnavailable => "OpenAiUnavailableException",
            OpenAiResponsesGatewayException::InvalidOpenAiResponse => "InvalidOpenAiResponseException",
            OpenAiResponsesGatewayException::IncompleteOpenAiResponse => "IncompleteOpenAiResponseException",
            OpenAiResponsesGatewayException::OpenAiConnection => "OpenAiConnectionException",
            OpenAiResponsesGatewayException::PreviousResponseNotFound => "PreviousResponseNotFoundException",
        }
    }
}
