use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::example_app_facade::ExampleAppFacade;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::models::greeting::Greeting;
use crate::services::http::exception_handlers::application_exception_response::ApplicationExceptionResponse;
use crate::services::http::extractors::json_body::JsonBody;
use crate::services::http::extractors::path_parameters::PathParameters;
use crate::services::http::mappers::audit_event_response_mapper::AuditEventResponseMapper;
use crate::services::http::mappers::greeting_response_mapper::GreetingResponseMapper;
use crate::services::http::mappers::save_customer_request_mapper::SaveCustomerRequestMapper;
use crate::services::http::wire_models::audit_event_response::AuditEventListResponse;
use crate::services::http::wire_models::error_response::ErrorResponse;
use crate::services::http::wire_models::greeting_response::GreetingResponse;
use crate::services::http::wire_models::save_customer_request::SaveCustomerRequest;

// Each route maps the request into domain models, calls one facade operation, and maps the domain models it returns
// to a response model. Domain models never cross the wire in either direction. Each route also names the client
// errors it can answer with (`responses`), so they are part of the service's contract (openapi.json). The facade
// comes from the router's state (`State`), which holds the one facade the service created; nothing builds one.
pub struct CustomerRouter;

impl CustomerRouter {
    pub fn router() -> OpenApiRouter<Arc<ExampleAppFacade>> {
        OpenApiRouter::new()
            .routes(routes!(save_customer))
            .routes(routes!(create_customer_greeting))
            .routes(routes!(get_customer_audit_events))
    }
}

#[utoipa::path(
    put,
    path = "/customers/{customer_id}",
    params(("customer_id" = String, Path)),
    request_body = SaveCustomerRequest,
    responses(
        (status = NO_CONTENT, description = "The customer was saved"),
        (status = BAD_REQUEST, description = "The customer can't be saved as given", body = ErrorResponse),
        (status = UNPROCESSABLE_ENTITY, description = "The body does not fit the request model", body = ErrorResponse),
    ),
)]
async fn save_customer(
    State(example_app_facade): State<Arc<ExampleAppFacade>>,
    PathParameters(customer_id): PathParameters<String>,
    JsonBody(save_customer_request): JsonBody<SaveCustomerRequest>,
) -> Result<StatusCode, ApplicationExceptionResponse> {
    let customer: Customer = SaveCustomerRequestMapper::to_customer(customer_id, save_customer_request);

    example_app_facade.save_customer(&customer).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/customers/{customer_id}/greetings",
    params(("customer_id" = String, Path)),
    responses(
        (status = CREATED, description = "The customer was greeted", body = GreetingResponse),
        (status = NOT_FOUND, description = "No customer has this id", body = ErrorResponse),
    ),
)]
async fn create_customer_greeting(
    State(example_app_facade): State<Arc<ExampleAppFacade>>,
    PathParameters(customer_id): PathParameters<String>,
) -> Result<(StatusCode, Json<GreetingResponse>), ApplicationExceptionResponse> {
    let greeting: Greeting = example_app_facade.get_customer_greeting(&customer_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(GreetingResponseMapper::to_greeting_response(greeting)),
    ))
}

#[utoipa::path(
    get,
    path = "/customers/{customer_id}/audit-events",
    params(("customer_id" = String, Path)),
    responses((status = OK, description = "The customer's audit events, oldest first", body = AuditEventListResponse)),
)]
async fn get_customer_audit_events(
    State(example_app_facade): State<Arc<ExampleAppFacade>>,
    PathParameters(customer_id): PathParameters<String>,
) -> Result<Json<AuditEventListResponse>, ApplicationExceptionResponse> {
    let audit_events: Vec<AuditEvent> = example_app_facade.get_customer_audit_events(&customer_id).await?;

    Ok(Json(AuditEventResponseMapper::to_audit_event_list_response(
        audit_events,
    )))
}
