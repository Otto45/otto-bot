import { setTimeout } from "node:timers/promises";

import type { z } from "zod";

import { RetryDelayCalculator } from "#src/domain_container/managers/common/calculators/retry_delay_calculator.ts";
import { ValidationErrorFormatter } from "#src/domain_container/managers/common/formatters/validation_error_formatter.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidUserProfileResponseException } from "#src/domain_container/managers/exceptions/invalid_user_profile_response_exception.ts";
import { JsonPlaceholderRequestException } from "#src/domain_container/managers/exceptions/json_placeholder_request_exception.ts";
import { JsonPlaceholderUnavailableException } from "#src/domain_container/managers/exceptions/json_placeholder_unavailable_exception.ts";
import { UserProfileNotFoundException } from "#src/domain_container/managers/exceptions/user_profile_not_found_exception.ts";
import { UserProfileMapper } from "#src/domain_container/managers/gateways/json_placeholder/mappers/user_profile_mapper.ts";
import { UserResourceSchema } from "#src/domain_container/managers/gateways/json_placeholder/resource_models/user_resource.ts";
import type { UserResource } from "#src/domain_container/managers/gateways/json_placeholder/resource_models/user_resource.ts";
import { ValidatorUserResource } from "#src/domain_container/managers/gateways/json_placeholder/validators/validator_user_resource.ts";
import { ApplicationLogger } from "#src/domain_container/managers/logger/application_logger.ts";
import type { UserProfile } from "#src/domain_container/managers/models/user_profile.ts";

const HTTP_NOT_FOUND: number = 404;
// The statuses that mean JSON Placeholder failed on its side or is limiting requests for now.
const UNAVAILABLE_STATUSES: ReadonlySet<number> = new Set<number>([429, 500, 502, 503, 504]);

export class JsonPlaceholderGateway {
  private readonly fetch: typeof globalThis.fetch;
  private readonly baseUrl: string;
  private readonly timeoutMilliseconds: number;
  private readonly maxRetries: number;

  /**
   * Given its fetch (a seam's), the service's base URL, which it joins to each path, the configured timeout, and how
   * many times a request that failed for a reason that may pass is sent again.
   */
  public constructor(fetch: typeof globalThis.fetch, baseUrl: string, timeoutSeconds: number, maxRetries: number) {
    this.fetch = fetch;
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.timeoutMilliseconds = timeoutSeconds * 1000;
    this.maxRetries = maxRetries;
  }

  public async getUserProfile(userId: number): Promise<UserProfile> {
    const context: ContextualData = new ContextualData().withValue("user_id", userId);
    const response: Response = await this.get(`/users/${userId}`, "retrieving a user profile", context);
    if (response.status === HTTP_NOT_FOUND) {
      throw new UserProfileNotFoundException("The user profile was not found", context);
    }
    if (!response.ok) {
      throw JsonPlaceholderGateway.toRequestException(undefined, "retrieving a user profile", context, response.status);
    }
    let body: string;
    try {
      body = await response.text();
    } catch (error: unknown) {
      throw JsonPlaceholderGateway.toRequestException(error, "retrieving a user profile", context, undefined);
    }

    const userResource: UserResource = JsonPlaceholderGateway.deserializeUserResource(body, context);
    ValidatorUserResource.validateUserResource(userResource);
    return UserProfileMapper.toUserProfile(userResource);
  }

  /** fetch keeps no connection the gateway owns, so there is nothing to close. */
  public close(): void {}

  // A GET's response. A GET is safe to send twice, so one that fails for a reason that may pass (it couldn't reach the
  // service or timed out, an UNAVAILABLE_STATUSES answer) is sent again, up to maxRetries times, after a growing wait
  // or the one the service asks for; the last failure is a JsonPlaceholderUnavailableException.
  private async get(path: string, operationDescription: string, context: ContextualData): Promise<Response> {
    for (let retry: number = 0; ; retry++) {
      try {
        return await this.getOnce(path, operationDescription, context);
      } catch (error: unknown) {
        if (!(error instanceof JsonPlaceholderUnavailableException) || retry >= this.maxRetries) {
          throw error;
        }
        const retryAfterSeconds: unknown = error.context.get("retry_after_seconds");
        const delayMilliseconds: number = RetryDelayCalculator.getRetryDelayMilliseconds(
          retry + 1,
          typeof retryAfterSeconds === "number" ? retryAfterSeconds : undefined,
        );
        ApplicationLogger.warning(
          "JSON Placeholder failed a request; it is sent again",
          error.context.withValue("retry", retry + 1).withValue("delay_seconds", delayMilliseconds / 1000),
          error,
        );
        await setTimeout(delayMilliseconds);
      }
    }
  }

  private async getOnce(path: string, operationDescription: string, context: ContextualData): Promise<Response> {
    let response: Response;
    try {
      response = await this.fetch(this.baseUrl + path, { signal: AbortSignal.timeout(this.timeoutMilliseconds) });
    } catch (error: unknown) {
      // A request that couldn't reach the service, or timed out, may well get through if sent again.
      throw new JsonPlaceholderUnavailableException(
        `JSON Placeholder is unavailable while ${operationDescription}`,
        context.withValue("error_type", error instanceof Error ? error.name : "Error"),
        { cause: error },
      );
    }
    if (UNAVAILABLE_STATUSES.has(response.status)) {
      let exceptionContext: ContextualData = context
        .withValue("error_type", "StatusError")
        .withValue("status_code", response.status);
      // How long the service asks to be left before the request is sent again, in whole seconds.
      const retryAfterSeconds: number | undefined = RetryDelayCalculator.parseRetryAfter(
        response.headers.get("retry-after"),
      );
      if (retryAfterSeconds !== undefined) {
        exceptionContext = exceptionContext.withValue("retry_after_seconds", retryAfterSeconds);
      }
      throw new JsonPlaceholderUnavailableException(
        `JSON Placeholder is unavailable while ${operationDescription}`,
        exceptionContext,
      );
    }
    return response;
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
