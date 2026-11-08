from typing import Final, final
from urllib.parse import SplitResult, urlsplit

import asyncpg
import httpx

from example_app.managers.configuration_providers.configuration_source_providers.configuration_source_provider_environment import (  # noqa: E501
    ConfigurationSourceProviderEnvironment,
)
from example_app.managers.configuration_providers.configuration_source_providers.configuration_source_provider_protocol import (  # noqa: E501
    ConfigurationSourceProviderProtocol,
)
from example_app.managers.data_layer.managers.gateways.document_database.clients.in_memory_document_database_client import (  # noqa: E501
    InMemoryDocumentDatabaseClient,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_configuration_exception import InvalidConfigurationException
from example_app.managers.models.settings.document_database_settings import DocumentDatabaseSettings
from example_app.managers.models.settings.json_placeholder_settings import JsonPlaceholderSettings
from example_app.managers.models.settings.relational_database_settings import RelationalDatabaseSettings
from example_app.managers.service_locator.service_locator_seams_protocol import ServiceLocatorSeamsProtocol

# How long opening a connection may take before it fails.
_RELATIONAL_DATABASE_CONNECT_TIMEOUT_SECONDS: Final[float] = 10
# A connection unused for this long is closed, so a database that sleeps while nothing is connected to it can.
_RELATIONAL_DATABASE_IDLE_SECONDS: Final[float] = 60


# The production seams: the environment, the network and the database drivers.
@final
class ServiceLocatorSeams(ServiceLocatorSeamsProtocol):
    def create_configuration_source_provider(self) -> ConfigurationSourceProviderProtocol:
        return ConfigurationSourceProviderEnvironment()

    def create_json_placeholder_http_client(
        self, json_placeholder_settings: JsonPlaceholderSettings
    ) -> httpx.AsyncClient:
        return httpx.AsyncClient(
            base_url=json_placeholder_settings.base_url, timeout=json_placeholder_settings.timeout_seconds
        )

    def create_relational_database_pool(self, relational_database_settings: RelationalDatabaseSettings) -> asyncpg.Pool:
        # Only checked here: nothing connects until the gateway first sends a statement. The connection string itself is
        # never quoted, because it holds the password.
        connection_url: SplitResult = urlsplit(relational_database_settings.connection_string)
        if connection_url.scheme not in ("postgres", "postgresql"):
            raise InvalidConfigurationException(
                "Relational database configuration is invalid",
                ContextualData()
                .with_value("setting_name", "relational_database")
                .with_value("errors", "connection_string is not a Postgres connection string"),
            )

        # The pool is created inside the running event loop, which it binds to, but not started: the gateway starts it
        # (awaits it) when it first sends a statement. It keeps no connection open while the app is idle.
        return asyncpg.create_pool(
            relational_database_settings.connection_string,
            min_size=0,
            max_size=10,
            max_inactive_connection_lifetime=_RELATIONAL_DATABASE_IDLE_SECONDS,
            timeout=_RELATIONAL_DATABASE_CONNECT_TIMEOUT_SECONDS,
        )

    def create_document_database_client(
        self, document_database_settings: DocumentDatabaseSettings
    ) -> InMemoryDocumentDatabaseClient:
        return InMemoryDocumentDatabaseClient(document_database_settings.connection_string)
