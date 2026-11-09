use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;

// The statuses that mean the API failed on its side or is limiting requests for now.
const UNAVAILABLE_STATUSES: [i64; 5] = [429, 500, 502, 503, 504];
// The error types and codes that mean the same, whatever the status (a WebSocket error event may have none).
const UNAVAILABLE_ERRORS: [&str; 4] = [
    "server_error",
    "rate_limit_exceeded",
    "server_is_overloaded",
    "overloaded",
];

// Which exception an error the API answered with is: one sending the request again may fix (it failed on its side, it
// is limiting requests), or one it won't. A rate limit is not a quota: running out of credit is told apart by its code.
pub struct ApiErrorMapper;

impl ApiErrorMapper {
    pub fn to_exception_kind(
        status_code: Option<i64>,
        api_error_type: Option<&str>,
        api_error_code: Option<&str>,
    ) -> OpenAiResponsesGatewayException {
        if api_error_code == Some("previous_response_not_found") {
            return OpenAiResponsesGatewayException::PreviousResponseNotFound;
        }
        if api_error_code == Some("insufficient_quota") {
            return OpenAiResponsesGatewayException::OpenAiRequest;
        }
        let unavailable: bool = status_code.is_some_and(|status_code: i64| UNAVAILABLE_STATUSES.contains(&status_code))
            || [api_error_type, api_error_code]
                .into_iter()
                .flatten()
                .any(|error: &str| UNAVAILABLE_ERRORS.contains(&error));
        if unavailable {
            OpenAiResponsesGatewayException::OpenAiUnavailable
        } else {
            OpenAiResponsesGatewayException::OpenAiRequest
        }
    }
}
