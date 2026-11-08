from http import HTTPStatus
from typing import final

from fastapi import APIRouter

from example_app.managers.models.user_profile import UserProfile
from example_app.services.http.dependencies.example_app_facade_parameter import ExampleAppFacadeParameter
from example_app.services.http.mappers.user_profile_response_mapper import UserProfileResponseMapper
from example_app.services.http.wire_models.error_response import ErrorResponse
from example_app.services.http.wire_models.user_profile_response import UserProfileResponse


@final
class UserProfileRouter:
    @staticmethod
    def create_router() -> APIRouter:
        router: APIRouter = APIRouter()
        router.add_api_route(
            "/user-profiles/{user_id}",
            UserProfileRouter.get_user_profile,
            methods=["GET"],
            responses={
                HTTPStatus.OK: {"description": "The user profile"},
                HTTPStatus.NOT_FOUND: {"description": "No user profile has this id", "model": ErrorResponse},
                HTTPStatus.UNPROCESSABLE_CONTENT: {
                    "description": "The user id is not an integer",
                    "model": ErrorResponse,
                },
            },
        )
        return router

    @staticmethod
    async def get_user_profile(user_id: int, example_app_facade: ExampleAppFacadeParameter) -> UserProfileResponse:
        user_profile: UserProfile = await example_app_facade.get_user_profile(user_id)

        return UserProfileResponseMapper.to_user_profile_response(user_profile)
