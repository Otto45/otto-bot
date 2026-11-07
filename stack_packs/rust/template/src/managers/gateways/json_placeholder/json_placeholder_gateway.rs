use reqwest::StatusCode;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::gateways::json_placeholder::mappers::user_profile_mapper::UserProfileMapper;
use crate::managers::gateways::json_placeholder::resource_models::user_resource::UserResource;
use crate::managers::gateways::json_placeholder::validators::validator_user_resource::ValidatorUserResource;
use crate::managers::models::user_profile::UserProfile;

pub struct JsonPlaceholderGateway {
    http_client: reqwest::Client,
    // reqwest has no base URL of its own, so the gateway joins it to each path.
    base_url: String,
}

impl JsonPlaceholderGateway {
    pub fn new(http_client: reqwest::Client, base_url: String) -> Self {
        JsonPlaceholderGateway {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_user_profile(&self, user_id: i64) -> ApplicationResult<UserProfile> {
        let context: ContextualData = ContextualData::new().with("user_id", user_id);

        let response: reqwest::Response = self
            .http_client
            .get(format!("{}/users/{user_id}", self.base_url))
            .send()
            .await
            .map_err(|exception: reqwest::Error| {
                JsonPlaceholderGateway::to_request_exception(exception, "retrieving a user profile", &context)
            })?;

        if response.status() == StatusCode::NOT_FOUND {
            return Err(ApplicationBaseException::new(
                JsonPlaceholderGatewayException::UserProfileNotFound,
                "The user profile was not found",
                context,
            ));
        }

        let body: Vec<u8> = response
            .error_for_status()
            .map_err(|exception: reqwest::Error| {
                JsonPlaceholderGateway::to_request_exception(exception, "retrieving a user profile", &context)
            })?
            .bytes()
            .await
            .map_err(|exception: reqwest::Error| {
                JsonPlaceholderGateway::to_request_exception(exception, "retrieving a user profile", &context)
            })?
            .to_vec();

        let user_resource: UserResource = JsonPlaceholderGateway::deserialize_user_resource(&body, &context)?;
        ValidatorUserResource::validate_user_resource(&user_resource)?;
        Ok(UserProfileMapper::to_user_profile(user_resource))
    }

    /// The HTTP client holds no connection that outlives it: its pool is released when the last clone is dropped, so
    /// there is nothing to close explicitly, and nothing to await.
    pub fn close(&self) {}

    fn deserialize_user_resource(body: &[u8], context: &ContextualData) -> ApplicationResult<UserResource> {
        let to_exception = |errors: String| -> ApplicationBaseException {
            ApplicationBaseException::new(
                JsonPlaceholderGatewayException::InvalidUserProfileResponse,
                "User profile response could not be deserialized",
                context.clone().with("errors", errors),
            )
        };

        let mut deserializer: serde_json::Deserializer<serde_json::de::SliceRead<'_>> =
            serde_json::Deserializer::from_slice(body);
        let user_resource: UserResource = serde_path_to_error::deserialize(&mut deserializer).map_err(
            |exception: serde_path_to_error::Error<serde_json::Error>| {
                to_exception(ValidationErrorFormatter::format(&exception, "body"))
            },
        )?;
        // Anything after the JSON value makes the body as a whole malformed.
        deserializer.end().map_err(|exception: serde_json::Error| {
            to_exception(ValidationErrorFormatter::format_whole(&exception, "body"))
        })?;
        Ok(user_resource)
    }

    fn to_request_exception(
        http_error: reqwest::Error,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationBaseException {
        let mut exception_context: ContextualData = context
            .clone()
            .with("error_type", JsonPlaceholderGateway::describe_error_type(&http_error));

        if let Some(status_code) = http_error.status() {
            exception_context.insert("status_code", status_code.as_u16());
        }

        ApplicationBaseException::new(
            JsonPlaceholderGatewayException::JsonPlaceholderRequest,
            format!("JSON Placeholder request failed while {operation_description}"),
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
