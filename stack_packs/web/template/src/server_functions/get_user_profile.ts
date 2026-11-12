import { createServerFn } from "@tanstack/react-start";

import type { UserProfile } from "#src/domain_container/managers/models/user_profile.ts";
import { UserProfileResponseMapper } from "#src/mappers/user_profile_response_mapper.ts";
import { UserProfileRequestSchema } from "#src/wire_models/user_profile_request.ts";
import type { UserProfileResponse } from "#src/wire_models/user_profile_response.ts";

export const getUserProfile = createServerFn({ method: "GET" })
  .validator(UserProfileRequestSchema)
  .handler(async ({ data, context }): Promise<UserProfileResponse> => {
    const userProfile: UserProfile = await context.exampleAppFacade.getUserProfile(data.userId);
    return UserProfileResponseMapper.toUserProfileResponse(userProfile);
  });
