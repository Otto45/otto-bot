import { ValidatorString } from "#src/domain_container/managers/common/validators/validator_string.ts";
import type { ExampleConfig } from "#src/domain_container/managers/configuration_providers/config_models/example_config.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { MissingConfigurationException } from "#src/domain_container/managers/exceptions/missing_configuration_exception.ts";

export class ValidatorExample {
  public static validateExampleConfig(config: ExampleConfig): void {
    const errors: string[] = [ValidatorString.validate("greeting_prefix", config.greetingPrefix)].filter(
      (error: string | undefined): error is string => error !== undefined,
    );

    if (errors.length > 0) {
      throw new MissingConfigurationException(
        "Example configuration is invalid",
        new ContextualData().withValue("setting_name", "example").withValue("errors", errors.join("; ")),
      );
    }
  }
}
