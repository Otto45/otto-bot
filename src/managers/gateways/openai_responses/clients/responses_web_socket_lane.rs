use crate::managers::gateways::openai_responses::clients::lane_continuation::LaneContinuation;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connection::ResponsesWebSocketConnection;
use crate::managers::models::model_request::ModelRequest;

/// A lane: its own connection, and the response that connection can continue. A lane serves one request at a time.
#[derive(Default)]
pub struct ResponsesWebSocketLane {
    pub connection: Option<ResponsesWebSocketConnection>,
    pub continuation: Option<LaneContinuation>,
    /// Set while a request is under way. Still set when the lane is next used means the request was abandoned (its
    /// run was cancelled), and its connection may still be sending that response, so it can't be used again.
    pub request_in_flight: bool,
}

impl ResponsesWebSocketLane {
    /// The response to continue and how many of the request's items it already covers, when the request opens with
    /// exactly the lane's continuation.
    pub fn get_continuation_point(&self, model_request: &ModelRequest) -> Option<(String, usize)> {
        let continuation: &LaneContinuation = self.continuation.as_ref()?;
        let covered_count: usize = continuation.items.len();
        let continues: bool = continuation.prompt_layers == model_request.prompt_layers
            && model_request.items.len() > covered_count
            && model_request.items[..covered_count] == continuation.items[..];
        continues.then(|| (continuation.response_id.clone(), covered_count))
    }
}
