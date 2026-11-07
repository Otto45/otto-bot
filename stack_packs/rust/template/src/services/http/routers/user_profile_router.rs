use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::example_app_facade::ExampleAppFacade;
use crate::managers::models::user_profile::UserProfile;
use crate::services::http::exception_handlers::application_exception_response::ApplicationExceptionResponse;
use crate::services::http::extractors::path_parameters::PathParameters;
use crate::services::http::mappers::user_profile_response_mapper::UserProfileResponseMapper;
use crate::services::http::wire_models::error_response::ErrorResponse;
use crate::services::http::wire_models::user_profile_response::UserProfileResponse;

pub struct UserProfileRouter;

impl UserProfileRouter {
    pub fn router() -> OpenApiRouter<Arc<ExampleAppFacade>> {
        OpenApiRouter::new().routes(routes!(get_user_profile))
    }
}

#[utoipa::path(
    get,
    path = "/user-profiles/{user_id}",
    params(("user_id" = i64, Path)),
    responses(
        (status = OK, description = "The user profile", body = UserProfileResponse),
        (status = BAD_REQUEST, description = "The user id is not an integer", body = ErrorResponse),
        (status = NOT_FOUND, description = "No user profile has this id", body = ErrorResponse),
    ),
)]
async fn get_user_profile(
    State(example_app_facade): State<Arc<ExampleAppFacade>>,
    PathParameters(user_id): PathParameters<i64>,
) -> Result<Json<UserProfileResponse>, ApplicationExceptionResponse> {
    let user_profile: UserProfile = example_app_facade.get_user_profile(user_id).await?;

    Ok(Json(UserProfileResponseMapper::to_user_profile_response(user_profile)))
}
