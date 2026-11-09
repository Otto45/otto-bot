import type { UserProfile } from "#src/managers/models/user_profile.ts";
import type { UserProfileResponse } from "#src/services/http/wire_models/user_profile_response.ts";

export class UserProfileResponseMapper {
  public static toUserProfileResponse(userProfile: UserProfile): UserProfileResponse {
    return {
      id: userProfile.id,
      name: userProfile.name,
      username: userProfile.username,
      email: userProfile.email,
      city: userProfile.city,
      company_name: userProfile.companyName,
    };
  }
}
