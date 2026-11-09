import type { DocumentDatabaseConfig } from "#src/managers/configuration_providers/config_models/document_database_config.ts";
import type { ExampleConfig } from "#src/managers/configuration_providers/config_models/example_config.ts";
import type { JsonPlaceholderConfig } from "#src/managers/configuration_providers/config_models/json_placeholder_config.ts";
import type { RelationalDatabaseConfig } from "#src/managers/configuration_providers/config_models/relational_database_config.ts";

export interface ConfigurationSourceProviderProtocol {
  getExampleConfig(): ExampleConfig;

  getRelationalDatabaseConfig(): RelationalDatabaseConfig;

  getDocumentDatabaseConfig(): DocumentDatabaseConfig;

  getJsonPlaceholderConfig(): JsonPlaceholderConfig;
}
