import pg from "pg";

import { ConfigurationSourceProviderEnvironment } from "#src/managers/configuration_providers/configuration_source_providers/configuration_source_provider_environment.ts";
import type { ConfigurationSourceProviderProtocol } from "#src/managers/configuration_providers/configuration_source_providers/configuration_source_provider_protocol.ts";
import { InMemoryDocumentDatabaseClient } from "#src/managers/data_layer/managers/gateways/document_database/clients/in_memory_document_database_client.ts";
import { ContextualData } from "#src/managers/exceptions/contextual_data.ts";
import { InvalidConfigurationException } from "#src/managers/exceptions/invalid_configuration_exception.ts";
import type { DocumentDatabaseSettings } from "#src/managers/models/settings/document_database_settings.ts";
import type { RelationalDatabaseSettings } from "#src/managers/models/settings/relational_database_settings.ts";
import type { ServiceLocatorSeamsProtocol } from "#src/managers/service_locator/service_locator_seams_protocol.ts";

// How long a statement waits for a connection: long enough for a database that sleeps to wake, short enough that one
// that can't be reached fails the request rather than hanging it.
const RELATIONAL_DATABASE_CONNECTION_TIMEOUT_MILLISECONDS: number = 10_000;
// A connection unused for this long is closed, so a database that sleeps while nothing is connected to it can.
const RELATIONAL_DATABASE_IDLE_MILLISECONDS: number = 60_000;

// The production seams: the environment, the network and the database drivers.
export class ServiceLocatorSeams implements ServiceLocatorSeamsProtocol {
  public createConfigurationSourceProvider(): ConfigurationSourceProviderProtocol {
    return new ConfigurationSourceProviderEnvironment();
  }

  // fetch takes its timeout with each request, so it needs no settings of its own.
  public createJsonPlaceholderFetch(): typeof globalThis.fetch {
    return globalThis.fetch;
  }

  public createRelationalDatabasePool(relationalDatabaseSettings: RelationalDatabaseSettings): pg.Pool {
    // Only checked here: the pool connects when the gateway first sends a statement. The connection string itself is
    // never quoted, because it holds the password.
    if (!/^postgres(?:ql)?:\/\//.test(relationalDatabaseSettings.connectionString)) {
      throw new InvalidConfigurationException(
        "Relational database configuration is invalid",
        new ContextualData()
          .withValue("setting_name", "relational_database")
          .withValue("errors", "connection_string is not a Postgres connection string"),
      );
    }
    // It keeps no connection open while the app is idle.
    return new pg.Pool({
      connectionString: relationalDatabaseSettings.connectionString,
      min: 0,
      max: 10,
      idleTimeoutMillis: RELATIONAL_DATABASE_IDLE_MILLISECONDS,
      connectionTimeoutMillis: RELATIONAL_DATABASE_CONNECTION_TIMEOUT_MILLISECONDS,
    });
  }

  public createDocumentDatabaseClient(
    documentDatabaseSettings: DocumentDatabaseSettings,
  ): InMemoryDocumentDatabaseClient {
    return new InMemoryDocumentDatabaseClient(documentDatabaseSettings.connectionString);
  }
}
