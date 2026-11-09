import { ValidatorString } from "#src/managers/common/validators/validator_string.ts";
import type { RelationalDatabaseConfig } from "#src/managers/configuration_providers/config_models/relational_database_config.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { MissingConfigurationException } from "#src/managers/exceptions/missing_configuration_exception.ts";

export class ValidatorRelationalDatabase {
  public static validateRelationalDatabaseConfig(config: RelationalDatabaseConfig): void {
    const errors: string[] = [ValidatorString.validate("connection_string", config.connectionString)].filter(
      (error: string | undefined): error is string => error !== undefined,
    );

    if (errors.length > 0) {
      throw new MissingConfigurationException(
        "Relational database configuration is invalid",
        new ContextualData().withValue("setting_name", "relational_database").withValue("errors", errors.join("; ")),
      );
    }
  }
}
