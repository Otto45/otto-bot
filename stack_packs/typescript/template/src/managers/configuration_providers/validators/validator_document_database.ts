import { ValidatorString } from "#src/managers/common/validators/validator_string.ts";
import type { DocumentDatabaseConfig } from "#src/managers/configuration_providers/config_models/document_database_config.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { MissingConfigurationException } from "#src/managers/exceptions/missing_configuration_exception.ts";

export class ValidatorDocumentDatabase {
  public static validateDocumentDatabaseConfig(config: DocumentDatabaseConfig): void {
    const errors: string[] = [ValidatorString.validate("connection_string", config.connectionString)].filter(
      (error: string | undefined): error is string => error !== undefined,
    );

    if (errors.length > 0) {
      throw new MissingConfigurationException(
        "Document database configuration is invalid",
        new ContextualData().withValue("setting_name", "document_database").withValue("errors", errors.join("; ")),
      );
    }
  }
}
