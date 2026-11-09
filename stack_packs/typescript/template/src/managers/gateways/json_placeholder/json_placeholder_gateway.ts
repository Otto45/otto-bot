import type { z } from "zod";

import { ValidationErrorFormatter } from "#src/managers/common/formatters/validation_error_formatter.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { InvalidUserProfileResponseException } from "#src/managers/exceptions/invalid_user_profile_response_exception.ts";
import { JsonPlaceholderRequestException } from "#src/managers/exceptions/json_placeholder_request_exception.ts";
import { UserProfileNotFoundException } from "#src/managers/exceptions/user_profile_not_found_exception.ts";
import { UserProfileMapper } from "#src/managers/gateways/json_placeholder/mappers/user_profile_mapper.ts";
import { UserResourceSchema } from "#src/managers/gateways/json_placeholder/resource_models/user_resource.ts";
import type { UserResource } from "#src/managers/gateways/json_placeholder/resource_models/user_resource.ts";
import { ValidatorUserResource } from "#src/managers/gateways/json_placeholder/validators/validator_user_resource.ts";
import type { UserProfile } from "#src/managers/models/user_profile.ts";

const HTTP_NOT_FOUND: number = 404;

export class JsonPlaceholderGateway {
  private readonly fetch: typeof globalThis.fetch;
  private readonly baseUrl: string;
  private readonly timeoutMilliseconds: number;

  /** Given its fetch (a seam's), the service's base URL, which it joins to each path, and the configured timeout. */
  public constructor(fetch: typeof globalThis.fetch, baseUrl: string, timeoutSeconds: number) {
    this.fetch = fetch;
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.timeoutMilliseconds = timeoutSeconds * 1000;
  }

  public async getUserProfile(userId: number): Promise<UserProfile> {
    const context: ContextualData = new ContextualData().withValue("user_id", userId);
    const body: string = await this.get(`/users/${userId}`, "retrieving a user profile", context);

    const userResource: UserResource = JsonPlaceholderGateway.deserializeUserResource(body, context);
    ValidatorUserResource.validateUserResource(userResource);
    return UserProfileMapper.toUserProfile(userResource);
  }

  /** fetch keeps no connection the gateway owns, so there is nothing to close. */
  public close(): void {}

  // The response's body; a 404 is the profile not being there, any other failure a request that failed.
  private async get(path: string, operationDescription: string, context: ContextualData): Promise<string> {
    let response: Response;
    try {
      response = await this.fetch(this.baseUrl + path, { signal: AbortSignal.timeout(this.timeoutMilliseconds) });
    } catch (error: unknown) {
      throw JsonPlaceholderGateway.toRequestException(error, operationDescription, context, undefined);
    }
    if (response.status === HTTP_NOT_FOUND) {
      throw new UserProfileNotFoundException("The user profile was not found", context);
    }
    if (!response.ok) {
      throw JsonPlaceholderGateway.toRequestException(undefined, operationDescription, context, response.status);
    }
    try {
      return await response.text();
    } catch (error: unknown) {
      throw JsonPlaceholderGateway.toRequestException(error, operationDescription, context, undefined);
    }
  }

  private static deserializeUserResource(body: string, context: ContextualData): UserResource {
    let json: unknown;
    try {
      json = JSON.parse(body);
    } catch (error: unknown) {
      throw new InvalidUserProfileResponseException(
        "User profile response could not be deserialized",
        context.withValue(
          "errors",
          ValidationErrorFormatter.formatWhole(error instanceof Error ? error : new Error(String(error)), "body"),
        ),
        { cause: error },
      );
    }
    const parsed: z.ZodSafeParseResult<UserResource> = UserResourceSchema.safeParse(json);
    if (!parsed.success) {
      throw new InvalidUserProfileResponseException(
        "User profile response could not be deserialized",
        context.withValue("errors", ValidationErrorFormatter.format(parsed.error, "body")),
      );
    }
    return parsed.data;
  }

  private static toRequestException(
    error: unknown,
    operationDescription: string,
    context: ContextualData,
    statusCode: number | undefined,
  ): JsonPlaceholderRequestException {
    let exceptionContext: ContextualData = context.withValue(
      "error_type",
      error instanceof Error ? error.name : "StatusError",
    );
    if (statusCode !== undefined) {
      exceptionContext = exceptionContext.withValue("status_code", statusCode);
    }
    return new JsonPlaceholderRequestException(
      `JSON Placeholder request failed while ${operationDescription}`,
      exceptionContext,
      { cause: error },
    );
  }
}
