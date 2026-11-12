import type pg from "pg";

import type { ConfigurationSourceProviderProtocol } from "#src/domain_container/managers/configuration_providers/configuration_source_providers/configuration_source_provider_protocol.ts";
import type { InMemoryDocumentDatabaseClient } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/clients/in_memory_document_database_client.ts";
import type { DocumentDatabaseSettings } from "#src/domain_container/managers/models/settings/document_database_settings.ts";
import type { JsonPlaceholderSettings } from "#src/domain_container/managers/models/settings/json_placeholder_settings.ts";
import type { RelationalDatabaseSettings } from "#src/domain_container/managers/models/settings/relational_database_settings.ts";

// The ServiceLocator's create* seams: the objects that reach outside the process, built from settings the locator hands
// in. The locator is given its seams, so whatever needs others builds the same production ServiceLocator with seams of
// its own, which call ServiceLocatorSeams for every seam they keep.
export interface ServiceLocatorSeamsProtocol {
  createConfigurationSourceProvider(): ConfigurationSourceProviderProtocol;

  createJsonPlaceholderFetch(jsonPlaceholderSettings: JsonPlaceholderSettings): typeof globalThis.fetch;

  createRelationalDatabasePool(relationalDatabaseSettings: RelationalDatabaseSettings): pg.Pool;

  createDocumentDatabaseClient(documentDatabaseSettings: DocumentDatabaseSettings): InMemoryDocumentDatabaseClient;
}
