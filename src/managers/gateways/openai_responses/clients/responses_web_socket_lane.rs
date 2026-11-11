use std::time::{Duration, Instant};

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
    /// When a connection could last not be opened: until `WEB_SOCKET_RETRY_INTERVAL` has passed, the lane's requests
    /// go over HTTP without trying again, so an endpoint with no WebSocket (or one that can't be reached) costs a lane
    /// one failed handshake, not one per request.
    pub connect_failed_on: Option<Instant>,
}

/// How long a lane that couldn't open a connection sends over HTTP before it tries WebSocket again.
const WEB_SOCKET_RETRY_INTERVAL: Duration = Duration::from_secs(10 * 60);

impl ResponsesWebSocketLane {
    /// Whether the lane's requests go over HTTP for now, because a connection couldn't be opened a short while ago.
    pub fn is_over_http(&self) -> bool {
        self.connect_failed_on
            .is_some_and(|connect_failed_on: Instant| connect_failed_on.elapsed() < WEB_SOCKET_RETRY_INTERVAL)
    }

    /// Records that a connection couldn't be opened, so the lane's requests go over HTTP for a while.
    pub fn record_connect_failed(&mut self) {
        self.connect_failed_on = Some(Instant::now());
    }

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
