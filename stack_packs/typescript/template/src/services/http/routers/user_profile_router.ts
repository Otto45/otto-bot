import { createRoute } from "@hono/zod-openapi";
import type { OpenAPIHono } from "@hono/zod-openapi";

import type { UserProfile } from "#src/managers/models/user_profile.ts";
import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import { UserProfileResponseMapper } from "#src/services/http/mappers/user_profile_response_mapper.ts";
import { ErrorResponseSchema } from "#src/services/http/wire_models/error_response.ts";
import { UserProfilePathParametersSchema } from "#src/services/http/wire_models/user_profile_path_parameters.ts";
import type { UserProfilePathParameters } from "#src/services/http/wire_models/user_profile_path_parameters.ts";
import { UserProfileResponseSchema } from "#src/services/http/wire_models/user_profile_response.ts";

export class UserProfileRouter {
  public static register(application: OpenAPIHono<HttpEnvironment>): void {
    application.openapi(
      createRoute({
        method: "get",
        path: "/user-profiles/{user_id}",
        request: { params: UserProfilePathParametersSchema },
        responses: {
          200: {
            description: "The user profile",
            content: { "application/json": { schema: UserProfileResponseSchema } },
          },
          404: {
            description: "No user profile has this id",
            content: { "application/json": { schema: ErrorResponseSchema } },
          },
          422: {
            description: "The user id is not an integer",
            content: { "application/json": { schema: ErrorResponseSchema } },
          },
        },
      }),
      async (context) => {
        const parameters: UserProfilePathParameters = context.req.valid("param");
        const userProfile: UserProfile = await context.get("exampleAppFacade").getUserProfile(parameters.user_id);

        return context.json(UserProfileResponseMapper.toUserProfileResponse(userProfile), 200);
      },
    );
  }
}
