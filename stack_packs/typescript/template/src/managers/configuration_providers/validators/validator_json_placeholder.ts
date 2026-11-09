import { ValidatorString } from "#src/managers/common/validators/validator_string.ts";
import type { JsonPlaceholderConfig } from "#src/managers/configuration_providers/config_models/json_placeholder_config.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { InvalidConfigurationException } from "#src/managers/exceptions/invalid_configuration_exception.ts";
import { MissingConfigurationException } from "#src/managers/exceptions/missing_configuration_exception.ts";

// More retries than this would hold a request for minutes on an outage that is no longer brief.
const MAX_RETRIES_LIMIT: number = 10;

export class ValidatorJsonPlaceholder {
  public static validateJsonPlaceholderConfig(config: JsonPlaceholderConfig): void {
    const errors: string[] = [];

    const baseUrlError: string | undefined = ValidatorString.validate("base_url", config.baseUrl);
    if (baseUrlError !== undefined) {
      errors.push(baseUrlError);
    }
    if (config.timeoutSeconds === undefined) {
      errors.push("timeout_seconds is missing");
    }
    if (config.maxRetries === undefined) {
      errors.push("max_retries is missing");
    }

    if (errors.length > 0) {
      throw new MissingConfigurationException(
        "JSON Placeholder configuration is invalid",
        new ContextualData().withValue("setting_name", "json_placeholder").withValue("errors", errors.join("; ")),
      );
    }

    if (
      config.maxRetries !== undefined &&
      (!Number.isInteger(config.maxRetries) || config.maxRetries < 0 || config.maxRetries > MAX_RETRIES_LIMIT)
    ) {
      throw new InvalidConfigurationException(
        "JSON Placeholder configuration is invalid",
        new ContextualData()
          .withValue("setting_name", "json_placeholder")
          .withValue("errors", `max_retries is not a whole number between 0 and ${MAX_RETRIES_LIMIT}`),
      );
    }

    // NaN is not positive either; an infinite timeout is no timeout.
    const timeoutSeconds: number | undefined = config.timeoutSeconds;
    let timeoutError: string | undefined = undefined;
    if (timeoutSeconds !== undefined && (Number.isNaN(timeoutSeconds) || timeoutSeconds <= 0)) {
      timeoutError = "timeout_seconds is not positive";
    } else if (timeoutSeconds !== undefined && !Number.isFinite(timeoutSeconds)) {
      timeoutError = "timeout_seconds is not finite";
    }
    if (timeoutError !== undefined) {
      throw new InvalidConfigurationException(
        "JSON Placeholder configuration is invalid",
        new ContextualData().withValue("setting_name", "json_placeholder").withValue("errors", timeoutError),
      );
    }
  }
}
