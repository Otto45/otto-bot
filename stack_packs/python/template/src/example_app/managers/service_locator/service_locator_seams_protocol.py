from typing import Protocol

import asyncpg
import httpx

from example_app.managers.configuration_providers.configuration_source_providers.configuration_source_provider_protocol import (  # noqa: E501
    ConfigurationSourceProviderProtocol,
)
from example_app.managers.data_layer.managers.gateways.document_database.clients.in_memory_document_database_client import (  # noqa: E501
    InMemoryDocumentDatabaseClient,
)
from example_app.managers.models.settings.document_database_settings import DocumentDatabaseSettings
from example_app.managers.models.settings.json_placeholder_settings import JsonPlaceholderSettings
from example_app.managers.models.settings.relational_database_settings import RelationalDatabaseSettings


# The ServiceLocator's create_* seams: the objects that reach outside the process, built from settings the locator
# hands in. The locator is given its seams, so whatever needs others builds the same production ServiceLocator with
# seams of its own, which call ServiceLocatorSeams for every seam they keep.
class ServiceLocatorSeamsProtocol(Protocol):
    def create_configuration_source_provider(self) -> ConfigurationSourceProviderProtocol: ...

    def create_json_placeholder_http_client(
        self, json_placeholder_settings: JsonPlaceholderSettings
    ) -> httpx.AsyncClient: ...

    def create_relational_database_pool(
        self, relational_database_settings: RelationalDatabaseSettings
    ) -> asyncpg.Pool: ...

    def create_document_database_client(
        self, document_database_settings: DocumentDatabaseSettings
    ) -> InMemoryDocumentDatabaseClient: ...
