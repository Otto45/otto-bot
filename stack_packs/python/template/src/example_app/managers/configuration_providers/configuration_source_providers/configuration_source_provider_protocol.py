from typing import Protocol

from example_app.managers.configuration_providers.config_models.document_database_config import DocumentDatabaseConfig
from example_app.managers.configuration_providers.config_models.example_config import ExampleConfig
from example_app.managers.configuration_providers.config_models.json_placeholder_config import JsonPlaceholderConfig
from example_app.managers.configuration_providers.config_models.relational_database_config import (
    RelationalDatabaseConfig,
)


class ConfigurationSourceProviderProtocol(Protocol):
    def get_example_config(self) -> ExampleConfig: ...

    def get_relational_database_config(self) -> RelationalDatabaseConfig: ...

    def get_document_database_config(self) -> DocumentDatabaseConfig: ...

    def get_json_placeholder_config(self) -> JsonPlaceholderConfig: ...
