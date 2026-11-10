use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, RETRY_AFTER};
use serde_json::Value;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::{ContextValue, ContextualData};
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::clients::lane_continuation::LaneContinuation;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connection::ResponsesWebSocketConnection;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_connector::ResponsesWebSocketConnector;
use crate::managers::gateways::openai_responses::clients::responses_web_socket_lane::ResponsesWebSocketLane;
use crate::managers::gateways::openai_responses::mappers::api_error_mapper::ApiErrorMapper;
use crate::managers::gateways::openai_responses::mappers::compacted_conversation_mapper::CompactedConversationMapper;
use crate::managers::gateways::openai_responses::mappers::model_response_mapper::ModelResponseMapper;
use crate::managers::gateways::openai_responses::request_builders::response_request_builder::ResponseRequestBuilder;
use crate::managers::gateways::openai_responses::resource_models::compaction_resource::CompactionResource;
use crate::managers::gateways::openai_responses::resource_models::error_response_resource::{
    ErrorResource, ErrorResponseResource,
};
use crate::managers::gateways::openai_responses::resource_models::response_resource::ResponseResource;
use crate::managers::gateways::openai_responses::validators::validator_compaction_resource::ValidatorCompactionResource;
use crate::managers::gateways::openai_responses::validators::validator_response_resource::ValidatorResponseResource;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::logger::log_level::LogLevel;
use crate::managers::models::compacted_conversation::CompactedConversation;
use crate::managers::models::conversation_item::ConversationItem;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;
use crate::managers::models::openai_transport::OpenAiTransport;

// How long the gateway waits before sending a request the API failed again: doubling from the first, up to the
// longest, with up to a quarter of a second added at random so requests that failed together aren't sent together.
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(1);
const LONGEST_RETRY_DELAY: Duration = Duration::from_secs(20);
const MAX_RETRY_JITTER_MILLISECONDS: u128 = 250;
// The longest the API's own Retry-After is waited for; asking for longer is treated as this.
const LONGEST_RETRY_AFTER: Duration = Duration::from_secs(60);

// Each lane behind its own lock, which a request holds across its await points: one request per lane at a time.
type Lanes = Mutex<HashMap<String, Arc<tokio::sync::Mutex<ResponsesWebSocketLane>>>>;

/// The OpenAI Responses API. Requests are stateless (`store=false`): the harness keeps every conversation itself, and
/// every ModelRequest carries a lane's whole conversation.
///
/// Over HTTP, each request sends it all. Over WebSocket, each lane has its own connection, which holds the lane's
/// latest response in memory: a request that only adds to the conversation that response covers continues it
/// (`previous_response_id`) and sends just the new items. When the connection no longer holds it (evicted, or a new
/// connection), or the connection is lost, the request is sent once more with the whole conversation on a fresh
/// connection; its prefix is cached, so that costs little. If no connection can be opened, the request goes over HTTP.
///
/// A request the API failed on its side (an error status like 500 or 503, a rate limit, a response that failed with a
/// server error), or that couldn't reach it, is sent again, up to `max_retries` times, after a growing wait (or the
/// wait the API asks for), whichever transport it went by. A request the API refused for a reason sending it again
/// won't change (it is malformed, unauthorized, over a quota) is not.
pub struct OpenAiResponsesGateway {
    http_client: reqwest::Client,
    web_socket_connector: ResponsesWebSocketConnector,
    transport: OpenAiTransport,
    // reqwest has no base URL of its own, so the gateway joins it to each path.
    base_url: String,
    // Sent as the bearer token; never logged, never put in a context.
    api_key: String,
    model: String,
    max_retries: i64,
    lanes: Lanes,
}

impl OpenAiResponsesGateway {
    pub fn new(
        http_client: reqwest::Client,
        web_socket_connector: ResponsesWebSocketConnector,
        transport: OpenAiTransport,
        base_url: String,
        api_key: String,
        model: String,
        max_retries: i64,
    ) -> Self {
        OpenAiResponsesGateway {
            http_client,
            web_socket_connector,
            transport,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            max_retries,
            lanes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn create_response(&self, model_request: &ModelRequest) -> ApplicationResult<ModelResponse> {
        let context: ContextualData = self.to_context(model_request);
        self.send_with_retries(|| async {
            match self.transport {
                OpenAiTransport::Http => self.create_response_over_http(model_request, &context).await,
                OpenAiTransport::WebSocket => self.create_response_over_web_socket(model_request, &context).await,
            }
        })
        .await
    }

    /// Compacts the lane's conversation (`/responses/compact`, always over HTTP: it is one request, not a lane's
    /// next), into the items its requests send from now on in place of it. Retried like a response.
    pub async fn compact_conversation(&self, model_request: &ModelRequest) -> ApplicationResult<CompactedConversation> {
        let context: ContextualData = self.to_context(model_request).with("operation", "compaction");
        let body: Vec<u8> = OpenAiResponsesGateway::serialize_body(
            &ResponseRequestBuilder::build_compaction(&self.model, model_request)?,
            &context,
        )?;
        let prompt_layers_message: Value = ResponseRequestBuilder::to_prompt_layers_message(model_request);
        self.send_with_retries(|| async {
            let response_body: Vec<u8> = self
                .post("responses/compact", body.clone(), "compacting a conversation", &context)
                .await?;
            self.to_compacted_conversation(&response_body, &prompt_layers_message, &context)
        })
        .await
    }

    fn to_context(&self, model_request: &ModelRequest) -> ContextualData {
        ContextualData::new()
            .with("model", self.model.as_str())
            .with("service_tier", model_request.service_tier.name())
            .with("lane_id", model_request.lane_id.as_str())
    }

    // Sends with `send` until it succeeds, fails in a way sending again won't change, or `max_retries` retries have
    // failed, waiting longer before each.
    async fn send_with_retries<T, Send, Sent>(&self, mut send: Send) -> ApplicationResult<T>
    where
        Send: FnMut() -> Sent,
        Sent: std::future::Future<Output = ApplicationResult<T>>,
    {
        let mut retry: i64 = 0;
        loop {
            let created: ApplicationResult<T> = send().await;
            let exception: ApplicationBaseException = match created {
                Err(exception) if retry < self.max_retries && OpenAiResponsesGateway::is_worth_retrying(&exception) => {
                    exception
                }
                created => return created,
            };
            retry += 1;
            let delay: Duration = OpenAiResponsesGateway::get_retry_delay(retry, &exception);
            // Expected now and then: a warning, so an outage shows in the log; the run carries on if a retry succeeds.
            ApplicationLogger::warning(
                "The OpenAI API failed a request; it is sent again",
                exception
                    .context()
                    .clone()
                    .with("retry", retry)
                    .with("max_retries", self.max_retries)
                    .with("delay_seconds", delay.as_secs_f64()),
                Some(&exception),
            );
            tokio::time::sleep(delay).await;
        }
    }

    // A failure on the API's side, or a connection that couldn't be made or was lost twice in a row. A response that
    // took longer than the whole response timeout isn't: sending it again would likely take as long.
    fn is_worth_retrying(exception: &ApplicationBaseException) -> bool {
        match exception.kind() {
            ExceptionKind::OpenAiResponsesGateway(OpenAiResponsesGatewayException::OpenAiUnavailable) => true,
            ExceptionKind::OpenAiResponsesGateway(OpenAiResponsesGatewayException::OpenAiConnection) => {
                exception
                    .context()
                    .get("error_type")
                    .map(ToString::to_string)
                    .as_deref()
                    != Some("TimeoutError")
            }
            _ => false,
        }
    }

    // The `retry`th wait: what the API asked for (Retry-After, up to LONGEST_RETRY_AFTER), or doubling from
    // FIRST_RETRY_DELAY up to LONGEST_RETRY_DELAY, whichever is longer, with jitter.
    fn get_retry_delay(retry: i64, exception: &ApplicationBaseException) -> Duration {
        let doubling: u32 = u32::try_from(retry.saturating_sub(1)).unwrap_or(u32::MAX).min(16);
        let backoff: Duration = FIRST_RETRY_DELAY
            .saturating_mul(2_u32.saturating_pow(doubling))
            .min(LONGEST_RETRY_DELAY);
        let retry_after: Duration = exception
            .context()
            .get("retry_after_seconds")
            .and_then(|retry_after_seconds: &ContextValue| match retry_after_seconds {
                ContextValue::Integer(seconds) => u64::try_from(*seconds).ok(),
                _ => None,
            })
            .map_or(Duration::ZERO, Duration::from_secs)
            .min(LONGEST_RETRY_AFTER);
        let jitter: Duration = Duration::from_millis(
            u64::try_from(uuid::Uuid::new_v4().as_u128() % MAX_RETRY_JITTER_MILLISECONDS).unwrap_or(0),
        );
        backoff.max(retry_after) + jitter
    }

    /// Closes every lane's connection. Called when the app stops, after every run has ended.
    pub async fn close(&self) {
        let lanes: Vec<Arc<tokio::sync::Mutex<ResponsesWebSocketLane>>> = self
            .lock_lanes()
            .drain()
            .map(|(_, lane): (String, Arc<tokio::sync::Mutex<ResponsesWebSocketLane>>)| lane)
            .collect();
        for lane in lanes {
            let connection: Option<ResponsesWebSocketConnection> = lane.lock().await.connection.take();
            if let Some(connection) = connection {
                connection.close().await;
            }
        }
    }

    async fn create_response_over_http(
        &self,
        model_request: &ModelRequest,
        context: &ContextualData,
    ) -> ApplicationResult<ModelResponse> {
        let body: Vec<u8> = OpenAiResponsesGateway::serialize_body(
            &ResponseRequestBuilder::build(&self.model, model_request)?,
            context,
        )?;
        let response_body: Vec<u8> = self.post("responses", body, "creating a response", context).await?;
        OpenAiResponsesGateway::to_model_response(&response_body, context)
    }

    // POSTs the JSON body to the API's `path`; the body of a successful response, or the error status as an exception.
    async fn post(
        &self,
        path: &str,
        body: Vec<u8>,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<Vec<u8>> {
        let response: reqwest::Response = self
            .http_client
            .post(format!("{}/{path}", self.base_url))
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|exception: reqwest::Error| {
                OpenAiResponsesGateway::to_request_exception(exception, operation_description, context)
            })?;

        let status: reqwest::StatusCode = response.status();
        // How long the API asks to be left before the request is sent again (a rate limit's), in whole seconds.
        let retry_after_seconds: Option<i64> = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|retry_after: &reqwest::header::HeaderValue| retry_after.to_str().ok())
            .and_then(|retry_after: &str| retry_after.trim().parse::<i64>().ok());
        let response_body: Vec<u8> = response
            .bytes()
            .await
            .map_err(|exception: reqwest::Error| {
                OpenAiResponsesGateway::to_request_exception(exception, operation_description, context)
            })?
            .to_vec();

        if !status.is_success() {
            return Err(OpenAiResponsesGateway::to_status_exception(
                status,
                retry_after_seconds,
                &response_body,
                context,
            ));
        }
        Ok(response_body)
    }

    // The compacted conversation, deserialized, validated against the prompt layers it was sent, and mapped.
    fn to_compacted_conversation(
        &self,
        body: &[u8],
        prompt_layers_message: &Value,
        context: &ContextualData,
    ) -> ApplicationResult<CompactedConversation> {
        let mut deserializer: serde_json::Deserializer<serde_json::de::SliceRead<'_>> =
            serde_json::Deserializer::from_slice(body);
        let compaction_resource: CompactionResource = serde_path_to_error::deserialize(&mut deserializer).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::InvalidOpenAiResponse,
                    "The compacted conversation could not be deserialized",
                    context
                        .clone()
                        .with("errors", ValidationErrorFormatter::format(&exception, "body")),
                )
            },
        )?;
        ValidatorCompactionResource::validate_compaction_resource(&compaction_resource, prompt_layers_message)?;
        Ok(CompactedConversationMapper::to_compacted_conversation(
            compaction_resource,
            &self.model,
        ))
    }

    async fn create_response_over_web_socket(
        &self,
        model_request: &ModelRequest,
        context: &ContextualData,
    ) -> ApplicationResult<ModelResponse> {
        let lane: Arc<tokio::sync::Mutex<ResponsesWebSocketLane>> = self.get_lane(&model_request.lane_id);
        let mut lane: tokio::sync::MutexGuard<'_, ResponsesWebSocketLane> = lane.lock().await;

        // The lane's last request was abandoned midway (its run was cancelled): its connection may still be sending
        // that response, so it is not used again.
        if lane.request_in_flight {
            lane.connection = None;
            lane.continuation = None;
            lane.request_in_flight = false;
        }

        let mut resent: bool = false;
        loop {
            if lane
                .connection
                .as_ref()
                .is_none_or(|connection: &ResponsesWebSocketConnection| connection.is_expiring())
            {
                // A new connection holds no response to continue.
                lane.continuation = None;
                lane.connection = None;
                match self.web_socket_connector.connect().await {
                    Ok(connection) => lane.connection = Some(connection),
                    Err(exception) => {
                        ApplicationLogger::log_exception(&exception, LogLevel::Warning);
                        return self.create_response_over_http(model_request, context).await;
                    }
                }
            }

            let continuation_point: Option<(String, usize)> = lane.get_continuation_point(model_request);
            let body: Value = match continuation_point.as_ref() {
                Some((previous_response_id, covered_count)) => ResponseRequestBuilder::build_continuation(
                    &self.model,
                    model_request,
                    previous_response_id,
                    *covered_count,
                )?,
                None => ResponseRequestBuilder::build(&self.model, model_request)?,
            };

            ApplicationLogger::info(
                "Sending a model request over the lane's WebSocket",
                context
                    .clone()
                    .with("continues_previous_response", continuation_point.is_some())
                    .with(
                        "items_sent",
                        i64::try_from(
                            model_request.items.len()
                                - continuation_point
                                    .as_ref()
                                    .map_or(0, |(_, covered_count): &(String, usize)| *covered_count),
                        )
                        .unwrap_or(i64::MAX),
                    ),
            );
            lane.request_in_flight = true;
            let sent: ApplicationResult<String> = lane
                .connection
                .as_mut()
                .expect("connected above")
                .create_response(&body)
                .await;
            lane.request_in_flight = false;

            let exception: ApplicationBaseException = match sent {
                Ok(response_text) => {
                    return match OpenAiResponsesGateway::to_model_response(response_text.as_bytes(), context) {
                        Ok(model_response) => {
                            lane.continuation = Some(OpenAiResponsesGateway::to_lane_continuation(
                                model_request,
                                &model_response,
                            ));
                            Ok(model_response)
                        }
                        // The connection holds a response the conversation won't contain.
                        Err(exception) => {
                            lane.continuation = None;
                            Err(exception)
                        }
                    };
                }
                Err(exception) => exception,
            };

            // After any error the lane's response is evicted; after a lost connection, the connection is gone.
            lane.continuation = None;
            let connection_lost: bool = exception.kind()
                == ExceptionKind::OpenAiResponsesGateway(OpenAiResponsesGatewayException::OpenAiConnection);
            if connection_lost {
                lane.connection = None;
            }
            let retriable: bool = connection_lost
                || exception.kind()
                    == ExceptionKind::OpenAiResponsesGateway(OpenAiResponsesGatewayException::PreviousResponseNotFound);
            if !retriable || resent {
                return Err(exception);
            }
            // Expected now and then (an evicted response, a connection the API closed): logged, then the whole
            // conversation is sent again.
            ApplicationLogger::log_exception(&exception, LogLevel::Info);
            resent = true;
        }
    }

    fn get_lane(&self, lane_id: &str) -> Arc<tokio::sync::Mutex<ResponsesWebSocketLane>> {
        self.lock_lanes()
            .entry(lane_id.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(ResponsesWebSocketLane::default())))
            .clone()
    }

    fn lock_lanes(&self) -> MutexGuard<'_, HashMap<String, Arc<tokio::sync::Mutex<ResponsesWebSocketLane>>>> {
        self.lanes.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // What the connection now holds: the response, covering the request's prompt and items and the response's own.
    fn to_lane_continuation(model_request: &ModelRequest, model_response: &ModelResponse) -> LaneContinuation {
        let mut items: Vec<ConversationItem> =
            Vec::with_capacity(model_request.items.len() + model_response.output_items.len());
        items.extend(model_request.items.iter().cloned());
        items.extend(model_response.output_items.iter().cloned());
        LaneContinuation {
            response_id: model_response.id.clone(),
            prompt_layers: model_request.prompt_layers.clone(),
            items,
        }
    }

    fn serialize_body(body: &Value, context: &ContextualData) -> ApplicationResult<Vec<u8>> {
        serde_json::to_vec(body).map_err(|exception: serde_json::Error| {
            ApplicationBaseException::new(
                OpenAiResponsesGatewayException::OpenAiRequest,
                "The model request could not be serialized",
                context.clone(),
            )
            .with_source(exception)
        })
    }

    // The response object (an HTTP body, or a WebSocket event's response), deserialized, validated and mapped.
    fn to_model_response(body: &[u8], context: &ContextualData) -> ApplicationResult<ModelResponse> {
        let response_resource: ResponseResource = OpenAiResponsesGateway::deserialize_response_resource(body, context)?;
        ValidatorResponseResource::validate_response_resource(&response_resource)?;
        Ok(ModelResponseMapper::to_model_response(response_resource))
    }

    fn deserialize_response_resource(body: &[u8], context: &ContextualData) -> ApplicationResult<ResponseResource> {
        let to_exception = |errors: String| -> ApplicationBaseException {
            ApplicationBaseException::new(
                OpenAiResponsesGatewayException::InvalidOpenAiResponse,
                "The model's response could not be deserialized",
                context.clone().with("errors", errors),
            )
        };

        let mut deserializer: serde_json::Deserializer<serde_json::de::SliceRead<'_>> =
            serde_json::Deserializer::from_slice(body);
        let response_resource: ResponseResource = serde_path_to_error::deserialize(&mut deserializer).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                to_exception(ValidationErrorFormatter::format(&exception, "body"))
            },
        )?;
        // Anything after the JSON value makes the body as a whole malformed.
        deserializer.end().map_err(|exception: serde_json::Error| {
            to_exception(ValidationErrorFormatter::format_whole(&exception, "body"))
        })?;
        Ok(response_resource)
    }

    // An error status, described by the API's error type and code when its body has them. The body's message is not
    // kept: it can quote the request.
    fn to_status_exception(
        status: reqwest::StatusCode,
        retry_after_seconds: Option<i64>,
        body: &[u8],
        context: &ContextualData,
    ) -> ApplicationBaseException {
        let mut exception_context: ContextualData = context
            .clone()
            .with("error_type", "StatusError")
            .with("status_code", status.as_u16());
        if let Some(retry_after_seconds) = retry_after_seconds {
            exception_context.insert("retry_after_seconds", retry_after_seconds);
        }

        let error_resource: ErrorResource = serde_json::from_slice::<ErrorResponseResource>(body)
            .ok()
            .and_then(|error_response_resource: ErrorResponseResource| error_response_resource.error)
            .unwrap_or_default();
        let exception_kind: OpenAiResponsesGatewayException = ApiErrorMapper::to_exception_kind(
            Some(i64::from(status.as_u16())),
            error_resource.error_type.as_deref(),
            error_resource.code.as_deref(),
        );
        if let Some(api_error_type) = error_resource.error_type {
            exception_context.insert("api_error_type", api_error_type);
        }
        if let Some(api_error_code) = error_resource.code {
            exception_context.insert("api_error_code", api_error_code);
        }

        ApplicationBaseException::new(
            exception_kind,
            format!("The OpenAI API answered with status {}", status.as_u16()),
            exception_context,
        )
    }

    fn to_request_exception(
        http_error: reqwest::Error,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        let mut exception_context: ContextualData = context
            .clone()
            .with("error_type", OpenAiResponsesGateway::describe_error_type(&http_error));

        if let Some(status_code) = http_error.status() {
            exception_context.insert("status_code", status_code.as_u16());
        }

        // A request that couldn't reach the API, or timed out, may well get through if sent again.
        let exception_kind: OpenAiResponsesGatewayException = if http_error.is_connect() || http_error.is_timeout() {
            OpenAiResponsesGatewayException::OpenAiUnavailable
        } else {
            OpenAiResponsesGatewayException::OpenAiRequest
        };
        ApplicationBaseException::new(
            exception_kind,
            format!("OpenAI request failed while {operation_description}"),
            exception_context,
        )
        .with_source(http_error)
    }

    // reqwest has one error type; what kind of failure it was is asked of it.
    fn describe_error_type(http_error: &reqwest::Error) -> &'static str {
        if http_error.is_status() {
            "StatusError"
        } else if http_error.is_timeout() {
            "TimeoutError"
        } else if http_error.is_connect() {
            "ConnectError"
        } else if http_error.is_body() || http_error.is_decode() {
            "BodyError"
        } else if http_error.is_request() {
            "RequestError"
        } else {
            "Error"
        }
    }
}
