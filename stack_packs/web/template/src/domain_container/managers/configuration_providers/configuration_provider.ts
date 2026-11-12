import { ValidatedValue } from "#src/domain_container/managers/common/validators/validated_value.ts";
import type { DocumentDatabaseConfig } from "#src/domain_container/managers/configuration_providers/config_models/document_database_config.ts";
import type { ExampleConfig } from "#src/domain_container/managers/configuration_providers/config_models/example_config.ts";
import type { JsonPlaceholderConfig } from "#src/domain_container/managers/configuration_providers/config_models/json_placeholder_config.ts";
import type { RelationalDatabaseConfig } from "#src/domain_container/managers/configuration_providers/config_models/relational_database_config.ts";
import type { ConfigurationSourceProviderProtocol } from "#src/domain_container/managers/configuration_providers/configuration_source_providers/configuration_source_provider_protocol.ts";
import { ValidatorDocumentDatabase } from "#src/domain_container/managers/configuration_providers/validators/validator_document_database.ts";
import { ValidatorExample } from "#src/domain_container/managers/configuration_providers/validators/validator_example.ts";
import { ValidatorJsonPlaceholder } from "#src/domain_container/managers/configuration_providers/validators/validator_json_placeholder.ts";
import { ValidatorRelationalDatabase } from "#src/domain_container/managers/configuration_providers/validators/validator_relational_database.ts";
import type { DocumentDatabaseSettings } from "#src/domain_container/managers/models/settings/document_database_settings.ts";
import type { ExampleSettings } from "#src/domain_container/managers/models/settings/example_settings.ts";
import type { JsonPlaceholderSettings } from "#src/domain_container/managers/models/settings/json_placeholder_settings.ts";
import type { RelationalDatabaseSettings } from "#src/domain_container/managers/models/settings/relational_database_settings.ts";

// Loads, validates and maps every setting on construction, so malformed configuration fails the moment the provider is
// created, before anything that depends on it is built.
export class ConfigurationProvider {
  private readonly exampleSettings: ExampleSettings;
  private readonly relationalDatabaseSettings: RelationalDatabaseSettings;
  private readonly documentDatabaseSettings: DocumentDatabaseSettings;
  private readonly jsonPlaceholderSettings: JsonPlaceholderSettings;

  /** Requires its source: the ServiceLocator's seams are the only place that decides where configuration comes from. */
  public constructor(configurationSourceProvider: ConfigurationSourceProviderProtocol) {
    this.exampleSettings = ConfigurationProvider.loadExampleSettings(configurationSourceProvider);
    this.relationalDatabaseSettings = ConfigurationProvider.loadRelationalDatabaseSettings(configurationSourceProvider);
    this.documentDatabaseSettings = ConfigurationProvider.loadDocumentDatabaseSettings(configurationSourceProvider);
    this.jsonPlaceholderSettings = ConfigurationProvider.loadJsonPlaceholderSettings(configurationSourceProvider);
  }

  public getExampleSettings(): ExampleSettings {
    return this.exampleSettings;
  }

  public getRelationalDatabaseSettings(): RelationalDatabaseSettings {
    return this.relationalDatabaseSettings;
  }

  public getDocumentDatabaseSettings(): DocumentDatabaseSettings {
    return this.documentDatabaseSettings;
  }

  public getJsonPlaceholderSettings(): JsonPlaceholderSettings {
    return this.jsonPlaceholderSettings;
  }

  // Each loader maps a config only after its validator passed it, so each ValidatedValue.expect states a checked fact:
  // it cannot fail unless the validator is wrong.
  private static loadExampleSettings(sourceProvider: ConfigurationSourceProviderProtocol): ExampleSettings {
    const config: ExampleConfig = sourceProvider.getExampleConfig();
    ValidatorExample.validateExampleConfig(config);
    return { greetingPrefix: ValidatedValue.expect(config.greetingPrefix, "ValidatorExample") };
  }

  private static loadRelationalDatabaseSettings(
    sourceProvider: ConfigurationSourceProviderProtocol,
  ): RelationalDatabaseSettings {
    const config: RelationalDatabaseConfig = sourceProvider.getRelationalDatabaseConfig();
    ValidatorRelationalDatabase.validateRelationalDatabaseConfig(config);
    return { connectionString: ValidatedValue.expect(config.connectionString, "ValidatorRelationalDatabase") };
  }

  private static loadDocumentDatabaseSettings(
    sourceProvider: ConfigurationSourceProviderProtocol,
  ): DocumentDatabaseSettings {
    const config: DocumentDatabaseConfig = sourceProvider.getDocumentDatabaseConfig();
    ValidatorDocumentDatabase.validateDocumentDatabaseConfig(config);
    return { connectionString: ValidatedValue.expect(config.connectionString, "ValidatorDocumentDatabase") };
  }

  private static loadJsonPlaceholderSettings(
    sourceProvider: ConfigurationSourceProviderProtocol,
  ): JsonPlaceholderSettings {
    const config: JsonPlaceholderConfig = sourceProvider.getJsonPlaceholderConfig();
    ValidatorJsonPlaceholder.validateJsonPlaceholderConfig(config);
    return {
      baseUrl: ValidatedValue.expect(config.baseUrl, "ValidatorJsonPlaceholder"),
      timeoutSeconds: ValidatedValue.expect(config.timeoutSeconds, "ValidatorJsonPlaceholder"),
      maxRetries: ValidatedValue.expect(config.maxRetries, "ValidatorJsonPlaceholder"),
    };
  }
}
