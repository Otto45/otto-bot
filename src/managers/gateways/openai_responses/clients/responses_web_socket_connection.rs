use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;

pub type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

// The API ends a connection after 60 minutes; one this old is replaced before a request, so none is cut off midway.
const MAX_CONNECTION_AGE: Duration = Duration::from_secs(55 * 60);
// The events that end a response; each carries the response object, as the HTTP endpoint returns it.
const RESPONSE_END_EVENTS: [&str; 3] = ["response.completed", "response.incomplete", "response.failed"];

/// One WebSocket connection to the Responses API, used for one request at a time. It keeps, in memory, the latest
/// response it served, which the next request can continue with `previous_response_id`.
pub struct ResponsesWebSocketConnection {
    web_socket: WebSocket,
    opened_on: Instant,
    response_timeout: Duration,
}

impl ResponsesWebSocketConnection {
    pub fn new(web_socket: WebSocket, response_timeout: Duration) -> Self {
        ResponsesWebSocketConnection {
            web_socket,
            opened_on: Instant::now(),
            response_timeout,
        }
    }

    pub fn is_expiring(&self) -> bool {
        self.opened_on.elapsed() >= MAX_CONNECTION_AGE
    }

    /// Sends a `response.create` event with `body` and waits for the response to end. Returns the response object's
    /// JSON text. An error event is an exception: PreviousResponseNotFound when the response the body continues isn't
    /// held any more, OpenAiRequest for anything else the API refused; a connection that is lost (or a response that
    /// takes too long) is an OpenAiConnection, and the connection can't be used again.
    pub async fn create_response(&mut self, body: &Value) -> ApplicationResult<String> {
        let mut event: Value = body.clone();
        event["type"] = Value::from("response.create");
        self.web_socket
            .send(Message::Text(event.to_string().into()))
            .await
            .map_err(|exception: tokio_tungstenite::tungstenite::Error| {
                ResponsesWebSocketConnection::to_connection_lost(Some(exception))
            })?;

        tokio::time::timeout(self.response_timeout, self.receive_response())
            .await
            .map_err(|_: tokio::time::error::Elapsed| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiConnection,
                    "The OpenAI API took too long to respond",
                    ContextualData::new().with("error_type", "TimeoutError"),
                )
            })?
    }

    /// Closes the connection; there is nothing to do if it is already gone.
    pub async fn close(mut self) {
        self.web_socket.close(None).await.ok();
    }

    async fn receive_response(&mut self) -> ApplicationResult<String> {
        while let Some(message) = self.web_socket.next().await {
            let message: Message = message.map_err(|exception: tokio_tungstenite::tungstenite::Error| {
                ResponsesWebSocketConnection::to_connection_lost(Some(exception))
            })?;
            let text: String = match message {
                Message::Text(text) => text.to_string(),
                Message::Close(_) => return Err(ResponsesWebSocketConnection::to_connection_lost(None)),
                // Pings are answered by the library; nothing else carries an event.
                _ => continue,
            };
            let event: Value = serde_json::from_str(&text).map_err(|exception: serde_json::Error| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::InvalidOpenAiResponse,
                    "The OpenAI API sent an event that is not JSON",
                    ContextualData::new(),
                )
                .with_source(exception)
            })?;

            match event.get("type").and_then(Value::as_str) {
                Some(event_type) if RESPONSE_END_EVENTS.contains(&event_type) => {
                    return Ok(event.get("response").cloned().unwrap_or(Value::Null).to_string());
                }
                Some("error") => return Err(ResponsesWebSocketConnection::to_error_exception(&event)),
                // Progress (created, deltas, items done): the response object at the end has all of it.
                _ => {}
            }
        }
        Err(ResponsesWebSocketConnection::to_connection_lost(None))
    }

    // Only the error's type, code and param are kept: its message can quote the request.
    fn to_error_exception(event: &Value) -> ApplicationBaseException {
        let error: &Value = event.get("error").unwrap_or(&Value::Null);
        let code: Option<&str> = error.get("code").and_then(Value::as_str);
        let mut context: ContextualData = ContextualData::new().with("error_type", "StatusError");
        if let Some(status_code) = event.get("status").and_then(Value::as_i64) {
            context.insert("status_code", status_code);
        }
        for (key, field) in [
            ("api_error_type", "type"),
            ("api_error_code", "code"),
            ("api_error_param", "param"),
        ] {
            if let Some(value) = error.get(field).and_then(Value::as_str) {
                context.insert(key, value);
            }
        }

        if code == Some("previous_response_not_found") {
            return ApplicationBaseException::new(
                OpenAiResponsesGatewayException::PreviousResponseNotFound,
                "The response to continue is no longer held by the connection",
                context,
            );
        }
        ApplicationBaseException::new(
            OpenAiResponsesGatewayException::OpenAiRequest,
            "The OpenAI API refused the request",
            context,
        )
    }

    fn to_connection_lost(exception: Option<tokio_tungstenite::tungstenite::Error>) -> ApplicationBaseException {
        let connection_lost: ApplicationBaseException = ApplicationBaseException::new(
            OpenAiResponsesGatewayException::OpenAiConnection,
            "The WebSocket connection to the OpenAI API was lost",
            ContextualData::new().with("error_type", "ConnectionLost"),
        );
        match exception {
            Some(exception) => connection_lost.with_source(exception),
            None => connection_lost,
        }
    }
}
