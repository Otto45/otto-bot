use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use tokio_tungstenite::Connector;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::{HeaderValue, Request};

use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connection::{
    ResponsesWebSocketConnection, WebSocket,
};

/// Opens WebSocket connections to the Responses API: the deepest object of the gateway's WebSocket transport, built
/// by the ServiceLocator's seam, the way the HTTP transport's reqwest::Client is.
#[derive(Clone)]
pub struct ResponsesWebSocketConnector {
    url: String,
    // Sent in the handshake's Authorization header; never logged, never put in a context.
    api_key: String,
    // The TLS configuration every wss handshake uses (an ws URL, as a local stand-in has, ignores it): built by the
    // seam with its crypto provider named, so the handshake never falls back to rustls' process-wide default.
    tls_configuration: Arc<rustls::ClientConfig>,
    response_timeout: Duration,
    idle_timeout: Duration,
}

impl ResponsesWebSocketConnector {
    /// `base_url` is the API's HTTP base URL (e.g. https://api.openai.com/v1); the connector speaks to its
    /// /responses endpoint over wss (ws for an http URL).
    pub fn new(
        base_url: &str,
        api_key: String,
        tls_configuration: Arc<rustls::ClientConfig>,
        response_timeout: Duration,
        idle_timeout: Duration,
    ) -> Self {
        let base_url: &str = base_url.trim_end_matches('/');
        let web_socket_base_url: String = match base_url.strip_prefix("https://") {
            Some(rest) => format!("wss://{rest}"),
            None => match base_url.strip_prefix("http://") {
                Some(rest) => format!("ws://{rest}"),
                None => base_url.to_string(),
            },
        };
        ResponsesWebSocketConnector {
            url: format!("{web_socket_base_url}/responses"),
            api_key,
            tls_configuration,
            response_timeout,
            idle_timeout,
        }
    }

    pub async fn connect(&self) -> ApplicationResult<ResponsesWebSocketConnection> {
        let context: ContextualData = ContextualData::new().with("url", self.url.as_str());
        let mut request: Request<()> =
            self.url
                .as_str()
                .into_client_request()
                .map_err(|exception: tokio_tungstenite::tungstenite::Error| {
                    ResponsesWebSocketConnector::to_connection_exception(exception, &context)
                })?;
        let authorization: HeaderValue = HeaderValue::from_str(&format!("Bearer {}", self.api_key)).map_err(
            |_: tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiConnection,
                    "The OpenAI API key can't be sent in a header",
                    context.clone(),
                )
            },
        )?;
        request.headers_mut().insert("Authorization", authorization);

        let (web_socket, _): (WebSocket, tokio_tungstenite::tungstenite::handshake::client::Response) =
            tokio::time::timeout(
                self.response_timeout,
                tokio_tungstenite::connect_async_tls_with_config(
                    request,
                    None,
                    false,
                    Some(Connector::Rustls(self.tls_configuration.clone())),
                ),
            )
            .await
            .map_err(|_: tokio::time::error::Elapsed| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiConnection,
                    "Connecting to the OpenAI API timed out",
                    context.clone().with("error_type", "TimeoutError"),
                )
            })?
            .map_err(|exception: tokio_tungstenite::tungstenite::Error| {
                ResponsesWebSocketConnector::to_connection_exception(exception, &context)
            })?;
        Ok(ResponsesWebSocketConnection::new(
            web_socket,
            self.response_timeout,
            self.idle_timeout,
        ))
    }

    fn to_connection_exception(
        exception: tokio_tungstenite::tungstenite::Error,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        let mut exception_context: ContextualData = context.clone().with("error_type", "ConnectError");
        if let tokio_tungstenite::tungstenite::Error::Http(response) = &exception {
            exception_context.insert("status_code", response.status().as_u16());
        }
        ApplicationBaseException::new(
            OpenAiResponsesGatewayException::OpenAiConnection,
            "The WebSocket connection to the OpenAI API could not be opened",
            exception_context,
        )
        .with_source(exception)
    }
}

// Written out rather than derived, so the API key never reaches a log or an error message.
impl fmt::Debug for ResponsesWebSocketConnector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponsesWebSocketConnector")
            .field("url", &self.url)
            .field("api_key", &"[REDACTED]")
            .field("tls_configuration", &"aws-lc-rs, platform verifier")
            .field("response_timeout", &self.response_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .finish()
    }
}
