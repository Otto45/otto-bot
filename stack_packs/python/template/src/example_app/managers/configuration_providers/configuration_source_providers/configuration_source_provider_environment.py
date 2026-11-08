import os
from typing import final

from example_app.managers.configuration_providers.config_models.document_database_config import DocumentDatabaseConfig
from example_app.managers.configuration_providers.config_models.example_config import ExampleConfig
from example_app.managers.configuration_providers.config_models.json_placeholder_config import JsonPlaceholderConfig
from example_app.managers.configuration_providers.config_models.relational_database_config import (
    RelationalDatabaseConfig,
)
from example_app.managers.configuration_providers.configuration_source_providers.configuration_source_provider_protocol import (  # noqa: E501
    ConfigurationSourceProviderProtocol,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_configuration_exception import InvalidConfigurationException


# The environment is read here and only here, inside these methods (never at import time), so a source that isn't the
# environment (e.g. a test's) is a different ConfigurationSourceProviderProtocol, not patched variables.
@final
class ConfigurationSourceProviderEnvironment(ConfigurationSourceProviderProtocol):
    def get_example_config(self) -> ExampleConfig:
        return ExampleConfig(
            greeting_prefix=ConfigurationSourceProviderEnvironment._get_string("EXAMPLE_APP_GREETING_PREFIX", "Hello"),
        )

    def get_relational_database_config(self) -> RelationalDatabaseConfig:
        return RelationalDatabaseConfig(
            # The app's database on the factory's development Postgres, which `dbmate up` creates and migrates.
            connection_string=ConfigurationSourceProviderEnvironment._get_string(
                "EXAMPLE_APP_RELATIONAL_DATABASE_CONNECTION_STRING",
                "postgresql://previews:previews@localhost:5433/example_app?sslmode=disable",
            ),
        )

    def get_document_database_config(self) -> DocumentDatabaseConfig:
        return DocumentDatabaseConfig(
            connection_string=ConfigurationSourceProviderEnvironment._get_string(
                "EXAMPLE_APP_DOCUMENT_DATABASE_CONNECTION_STRING", "in-memory://document"
            ),
        )

    def get_json_placeholder_config(self) -> JsonPlaceholderConfig:
        return JsonPlaceholderConfig(
            base_url=ConfigurationSourceProviderEnvironment._get_string(
                "EXAMPLE_APP_JSON_PLACEHOLDER_BASE_URL", "https://jsonplaceholder.typicode.com"
            ),
            timeout_seconds=ConfigurationSourceProviderEnvironment._get_float(
                "EXAMPLE_APP_JSON_PLACEHOLDER_TIMEOUT_SECONDS", "10"
            ),
        )

    @staticmethod
    def _get_string(environment_variable_name: str, default_value: str) -> str:
        return os.environ.get(environment_variable_name, default_value)

    @staticmethod
    def _get_float(environment_variable_name: str, default_value: str) -> float:
        value: str = ConfigurationSourceProviderEnvironment._get_string(environment_variable_name, default_value)
        try:
            return float(value.strip())
        except ValueError as error:
            raise InvalidConfigurationException(
                "Environment variable is not a number",
                ContextualData().with_value("environment_variable_name", environment_variable_name),
            ) from error
