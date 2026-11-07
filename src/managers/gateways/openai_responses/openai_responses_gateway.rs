use reqwest::header::CONTENT_TYPE;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::openai_responses_gateway_exception::OpenAiResponsesGatewayException;
use crate::managers::gateways::openai_responses::mappers::model_response_mapper::ModelResponseMapper;
use crate::managers::gateways::openai_responses::request_builders::response_request_builder::ResponseRequestBuilder;
use crate::managers::gateways::openai_responses::resource_models::error_response_resource::{
    ErrorResource, ErrorResponseResource,
};
use crate::managers::gateways::openai_responses::resource_models::response_resource::ResponseResource;
use crate::managers::gateways::openai_responses::validators::validator_response_resource::ValidatorResponseResource;
use crate::managers::models::model_request::ModelRequest;
use crate::managers::models::model_response::ModelResponse;

/// The OpenAI Responses API. Requests are stateless (`store=false`): the harness keeps every conversation itself.
pub struct OpenAiResponsesGateway {
    http_client: reqwest::Client,
    // reqwest has no base URL of its own, so the gateway joins it to each path.
    base_url: String,
    // Sent as the bearer token; never logged, never put in a context.
    api_key: String,
    model: String,
}

impl OpenAiResponsesGateway {
    pub fn new(http_client: reqwest::Client, base_url: String, api_key: String, model: String) -> Self {
        OpenAiResponsesGateway {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        }
    }

    pub async fn create_response(&self, model_request: &ModelRequest) -> ApplicationResult<ModelResponse> {
        let context: ContextualData = ContextualData::new()
            .with("model", self.model.as_str())
            .with("service_tier", model_request.service_tier.name());

        let body: Vec<u8> = serde_json::to_vec(&ResponseRequestBuilder::build(&self.model, model_request)).map_err(
            |exception: serde_json::Error| {
                ApplicationBaseException::new(
                    OpenAiResponsesGatewayException::OpenAiRequest,
                    "The model request could not be serialized",
                    context.clone(),
                )
                .with_source(exception)
            },
        )?;

        let response: reqwest::Response = self
            .http_client
            .post(format!("{}/responses", self.base_url))
            .bearer_auth(&self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|exception: reqwest::Error| {
                OpenAiResponsesGateway::to_request_exception(exception, "creating a response", &context)
            })?;

        let status: reqwest::StatusCode = response.status();
        let response_body: Vec<u8> = response
            .bytes()
            .await
            .map_err(|exception: reqwest::Error| {
                OpenAiResponsesGateway::to_request_exception(exception, "creating a response", &context)
            })?
            .to_vec();

        if !status.is_success() {
            return Err(OpenAiResponsesGateway::to_status_exception(
                status,
                &response_body,
                &context,
            ));
        }

        let response_resource: ResponseResource =
            OpenAiResponsesGateway::deserialize_response_resource(&response_body, &context)?;
        ValidatorResponseResource::validate_response_resource(&response_resource)?;
        Ok(ModelResponseMapper::to_model_response(response_resource))
    }

    /// The HTTP client holds no connection that outlives it: its pool is released when the last clone is dropped, so
    /// there is nothing to close explicitly, and nothing to await.
    pub fn close(&self) {}

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
        body: &[u8],
        context: &ContextualData,
    ) -> ApplicationBaseException {
        let mut exception_context: ContextualData = context
            .clone()
            .with("error_type", "StatusError")
            .with("status_code", status.as_u16());

        let error_resource: Option<ErrorResource> = serde_json::from_slice::<ErrorResponseResource>(body)
            .ok()
            .and_then(|error_response_resource: ErrorResponseResource| error_response_resource.error);
        if let Some(error_resource) = error_resource {
            if let Some(api_error_type) = error_resource.error_type {
                exception_context.insert("api_error_type", api_error_type);
            }
            if let Some(api_error_code) = error_resource.code {
                exception_context.insert("api_error_code", api_error_code);
            }
        }

        ApplicationBaseException::new(
            OpenAiResponsesGatewayException::OpenAiRequest,
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

        ApplicationBaseException::new(
            OpenAiResponsesGatewayException::OpenAiRequest,
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
