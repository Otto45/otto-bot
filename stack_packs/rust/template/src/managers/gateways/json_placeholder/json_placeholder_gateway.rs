use std::time::Duration;

use reqwest::StatusCode;
use reqwest::header::RETRY_AFTER;

use crate::managers::common::formatters::validation_error_formatter::ValidationErrorFormatter;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::{ContextValue, ContextualData};
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::exceptions::json_placeholder_gateway_exception::JsonPlaceholderGatewayException;
use crate::managers::gateways::json_placeholder::mappers::user_profile_mapper::UserProfileMapper;
use crate::managers::gateways::json_placeholder::resource_models::user_resource::UserResource;
use crate::managers::gateways::json_placeholder::validators::validator_user_resource::ValidatorUserResource;
use crate::managers::logger::application_logger::ApplicationLogger;
use crate::managers::models::user_profile::UserProfile;

// How long the gateway waits before sending a failed request again: doubling from the first, up to the longest, with up
// to a quarter of a second added at random so requests that failed together aren't sent together. The longest the
// service's own Retry-After is waited for; asking for longer is treated as this.
const FIRST_RETRY_DELAY: Duration = Duration::from_secs(1);
const LONGEST_RETRY_DELAY: Duration = Duration::from_secs(20);
const MAX_RETRY_JITTER_MILLISECONDS: u128 = 250;
const LONGEST_RETRY_AFTER: Duration = Duration::from_secs(60);
// The statuses that mean JSON Placeholder failed on its side or is limiting requests for now.
const UNAVAILABLE_STATUSES: [StatusCode; 5] = [
    StatusCode::TOO_MANY_REQUESTS,
    StatusCode::INTERNAL_SERVER_ERROR,
    StatusCode::BAD_GATEWAY,
    StatusCode::SERVICE_UNAVAILABLE,
    StatusCode::GATEWAY_TIMEOUT,
];

pub struct JsonPlaceholderGateway {
    http_client: reqwest::Client,
    // reqwest has no base URL of its own, so the gateway joins it to each path.
    base_url: String,
    // How many times a request that failed for a reason that may pass is sent again.
    max_retries: i64,
}

impl JsonPlaceholderGateway {
    pub fn new(http_client: reqwest::Client, base_url: String, max_retries: i64) -> Self {
        JsonPlaceholderGateway {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
            max_retries,
        }
    }

    pub async fn get_user_profile(&self, user_id: i64) -> ApplicationResult<UserProfile> {
        let context: ContextualData = ContextualData::new().with("user_id", user_id);

        let (status, body): (StatusCode, Vec<u8>) = self
            .get(&format!("/users/{user_id}"), "retrieving a user profile", &context)
            .await?;
        if status == StatusCode::NOT_FOUND {
            return Err(ApplicationBaseException::new(
                JsonPlaceholderGatewayException::UserProfileNotFound,
                "The user profile was not found",
                context,
            ));
        }
        if !status.is_success() {
            return Err(ApplicationBaseException::new(
                JsonPlaceholderGatewayException::JsonPlaceholderRequest,
                "JSON Placeholder request failed while retrieving a user profile",
                context
                    .with("error_type", "StatusError")
                    .with("status_code", status.as_u16()),
            ));
        }

        let user_resource: UserResource = JsonPlaceholderGateway::deserialize_user_resource(&body, &context)?;
        ValidatorUserResource::validate_user_resource(&user_resource)?;
        Ok(UserProfileMapper::to_user_profile(user_resource))
    }

    /// The HTTP client holds no connection that outlives it: its pool is released when the last clone is dropped, so
    /// there is nothing to close explicitly, and nothing to await.
    pub fn close(&self) {}

    // A GET's status and body. A GET is safe to send twice, so one that fails for a reason that may pass (a connection
    // that couldn't be made or timed out, an UNAVAILABLE_STATUSES answer) is sent again, up to max_retries times, after
    // a growing wait or the one the service asks for; the last failure is a JsonPlaceholderUnavailable.
    async fn get(
        &self,
        path: &str,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<(StatusCode, Vec<u8>)> {
        let mut retry: i64 = 0;
        loop {
            let exception: ApplicationBaseException = match self.get_once(path, operation_description, context).await {
                Err(exception)
                    if retry < self.max_retries
                        && exception.kind()
                            == ExceptionKind::JsonPlaceholderGateway(
                                JsonPlaceholderGatewayException::JsonPlaceholderUnavailable,
                            ) =>
                {
                    exception
                }
                answered => return answered,
            };
            retry += 1;
            let delay: Duration = JsonPlaceholderGateway::get_retry_delay(retry, &exception);
            ApplicationLogger::warning(
                "JSON Placeholder failed a request; it is sent again",
                exception
                    .context()
                    .clone()
                    .with("retry", retry)
                    .with("delay_seconds", delay.as_secs_f64()),
                Some(&exception),
            );
            tokio::time::sleep(delay).await;
        }
    }

    async fn get_once(
        &self,
        path: &str,
        operation_description: &str,
        context: &ContextualData,
    ) -> ApplicationResult<(StatusCode, Vec<u8>)> {
        let response: reqwest::Response = self
            .http_client
            .get(format!("{}{path}", self.base_url))
            .send()
            .await
            .map_err(|exception: reqwest::Error| {
                JsonPlaceholderGateway::to_request_exception(exception, operation_description, context)
            })?;

        let status: StatusCode = response.status();
        if UNAVAILABLE_STATUSES.contains(&status) {
            let mut exception_context: ContextualData = context
                .clone()
                .with("error_type", "StatusError")
                .with("status_code", status.as_u16());
            // How long the service asks to be left before the request is sent again, in whole seconds.
            if let Some(retry_after_seconds) = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|retry_after: &reqwest::header::HeaderValue| retry_after.to_str().ok())
                .and_then(|retry_after: &str| retry_after.trim().parse::<i64>().ok())
            {
                exception_context.insert("retry_after_seconds", retry_after_seconds);
            }
            return Err(ApplicationBaseException::new(
                JsonPlaceholderGatewayException::JsonPlaceholderUnavailable,
                format!("JSON Placeholder is unavailable while {operation_description}"),
                exception_context,
            ));
        }

        let body: Vec<u8> = response
            .bytes()
            .await
            .map_err(|exception: reqwest::Error| {
                JsonPlaceholderGateway::to_request_exception(exception, operation_description, context)
            })?
            .to_vec();
        Ok((status, body))
    }

    // The `retry`th wait: what the service asked for (Retry-After, up to LONGEST_RETRY_AFTER), or doubling from
    // FIRST_RETRY_DELAY up to LONGEST_RETRY_DELAY, whichever is longer, with jitter.
    fn get_retry_delay(retry: i64, exception: &ApplicationBaseException) -> Duration {
        let doubling: u32 = u32::try_from(retry.saturating_sub(1)).unwrap_or(u32::MAX).min(16);
        let backoff: Duration = FIRST_RETRY_DELAY
            .saturating_mul(2_u32.saturating_pow(doubling))
            .min(LONGEST_RETRY_DELAY);
        let retry_after: Duration = match exception.context().get("retry_after_seconds") {
            Some(ContextValue::Integer(seconds)) => u64::try_from(*seconds).map_or(Duration::ZERO, Duration::from_secs),
            _ => Duration::ZERO,
        }
        .min(LONGEST_RETRY_AFTER);
        let jitter: Duration = Duration::from_millis(
            u64::try_from(uuid::Uuid::new_v4().as_u128() % MAX_RETRY_JITTER_MILLISECONDS).unwrap_or(0),
        );
        backoff.max(retry_after) + jitter
    }

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

        // A request that couldn't reach the service, or timed out, may well get through if sent again.
        let exception_kind: JsonPlaceholderGatewayException = if http_error.is_connect() || http_error.is_timeout() {
            JsonPlaceholderGatewayException::JsonPlaceholderUnavailable
        } else {
            JsonPlaceholderGatewayException::JsonPlaceholderRequest
        };
        ApplicationBaseException::new(
            exception_kind,
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
