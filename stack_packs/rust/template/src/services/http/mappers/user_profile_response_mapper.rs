use crate::managers::models::user_profile::UserProfile;
use crate::services::http::wire_models::user_profile_response::UserProfileResponse;

pub struct UserProfileResponseMapper;

impl UserProfileResponseMapper {
    pub fn to_user_profile_response(user_profile: UserProfile) -> UserProfileResponse {
        UserProfileResponse {
            id: user_profile.id,
            name: user_profile.name,
            username: user_profile.username,
            email: user_profile.email,
            city: user_profile.city,
            company_name: user_profile.company_name,
        }
    }
}
