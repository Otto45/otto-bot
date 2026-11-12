import type { DocumentDatabaseConfig } from "#src/domain_container/managers/configuration_providers/config_models/document_database_config.ts";
import type { ExampleConfig } from "#src/domain_container/managers/configuration_providers/config_models/example_config.ts";
import type { JsonPlaceholderConfig } from "#src/domain_container/managers/configuration_providers/config_models/json_placeholder_config.ts";
import type { RelationalDatabaseConfig } from "#src/domain_container/managers/configuration_providers/config_models/relational_database_config.ts";
import type { ConfigurationSourceProviderProtocol } from "#src/domain_container/managers/configuration_providers/configuration_source_providers/configuration_source_provider_protocol.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidConfigurationException } from "#src/domain_container/managers/exceptions/invalid_configuration_exception.ts";

// The environment is read here and only here, inside these methods (never when a module loads), so a source that isn't
// the environment (e.g. a test's) is a different ConfigurationSourceProviderProtocol, not changed variables.
export class ConfigurationSourceProviderEnvironment implements ConfigurationSourceProviderProtocol {
  public getExampleConfig(): ExampleConfig {
    return { greetingPrefix: ConfigurationSourceProviderEnvironment.getString("EXAMPLE_APP_GREETING_PREFIX", "Hello") };
  }

  public getRelationalDatabaseConfig(): RelationalDatabaseConfig {
    return {
      // The app's database on the factory's development Postgres, which `dbmate up` creates and migrates.
      connectionString: ConfigurationSourceProviderEnvironment.getString(
        "EXAMPLE_APP_RELATIONAL_DATABASE_CONNECTION_STRING",
        "postgresql://previews:previews@localhost:5433/example_app?sslmode=disable",
      ),
    };
  }

  public getDocumentDatabaseConfig(): DocumentDatabaseConfig {
    return {
      connectionString: ConfigurationSourceProviderEnvironment.getString(
        "EXAMPLE_APP_DOCUMENT_DATABASE_CONNECTION_STRING",
        "in-memory://document",
      ),
    };
  }

  public getJsonPlaceholderConfig(): JsonPlaceholderConfig {
    return {
      baseUrl: ConfigurationSourceProviderEnvironment.getString(
        "EXAMPLE_APP_JSON_PLACEHOLDER_BASE_URL",
        "https://jsonplaceholder.typicode.com",
      ),
      timeoutSeconds: ConfigurationSourceProviderEnvironment.getNumber(
        "EXAMPLE_APP_JSON_PLACEHOLDER_TIMEOUT_SECONDS",
        "10",
      ),
      maxRetries: ConfigurationSourceProviderEnvironment.getNumber("EXAMPLE_APP_JSON_PLACEHOLDER_MAX_RETRIES", "3"),
    };
  }

  private static getString(environmentVariableName: string, defaultValue: string): string {
    return process.env[environmentVariableName] ?? defaultValue;
  }

  private static getNumber(environmentVariableName: string, defaultValue: string): number {
    const value: string = ConfigurationSourceProviderEnvironment.getString(
      environmentVariableName,
      defaultValue,
    ).trim();
    const number: number = Number(value);
    if (value === "" || Number.isNaN(number)) {
      throw new InvalidConfigurationException(
        "Environment variable is not a number",
        new ContextualData().withValue("environment_variable_name", environmentVariableName),
      );
    }
    return number;
  }
}
