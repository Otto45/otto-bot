from typing import final

from example_app.managers.models.user_profile import UserProfile
from example_app.services.http.wire_models.user_profile_response import UserProfileResponse


@final
class UserProfileResponseMapper:
    @staticmethod
    def to_user_profile_response(user_profile: UserProfile) -> UserProfileResponse:
        return UserProfileResponse(
            id=user_profile.id,
            name=user_profile.name,
            username=user_profile.username,
            email=user_profile.email,
            city=user_profile.city,
            company_name=user_profile.company_name,
        )
