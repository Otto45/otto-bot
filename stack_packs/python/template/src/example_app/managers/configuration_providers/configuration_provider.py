from typing import final

from example_app.managers.common.validators.validated_value import ValidatedValue
from example_app.managers.configuration_providers.config_models.document_database_config import DocumentDatabaseConfig
from example_app.managers.configuration_providers.config_models.example_config import ExampleConfig
from example_app.managers.configuration_providers.config_models.json_placeholder_config import JsonPlaceholderConfig
from example_app.managers.configuration_providers.config_models.relational_database_config import (
    RelationalDatabaseConfig,
)
from example_app.managers.configuration_providers.configuration_source_providers.configuration_source_provider_protocol import (  # noqa: E501
    ConfigurationSourceProviderProtocol,
)
from example_app.managers.configuration_providers.validators.validator_document_database import (
    ValidatorDocumentDatabase,
)
from example_app.managers.configuration_providers.validators.validator_example import ValidatorExample
from example_app.managers.configuration_providers.validators.validator_json_placeholder import ValidatorJsonPlaceholder
from example_app.managers.configuration_providers.validators.validator_relational_database import (
    ValidatorRelationalDatabase,
)
from example_app.managers.models.settings.document_database_settings import DocumentDatabaseSettings
from example_app.managers.models.settings.example_settings import ExampleSettings
from example_app.managers.models.settings.json_placeholder_settings import JsonPlaceholderSettings
from example_app.managers.models.settings.relational_database_settings import RelationalDatabaseSettings


# Loads, validates and maps every setting on construction, so malformed configuration fails the moment the provider is
# created, before anything that depends on it is built.
@final
class ConfigurationProvider:
    def __init__(self, configuration_source_provider: ConfigurationSourceProviderProtocol) -> None:
        """Requires its source: the ServiceLocator's seams are the only place that decides where configuration comes
        from."""
        self._example_settings: ExampleSettings = ConfigurationProvider._load_example_settings(
            configuration_source_provider
        )
        self._relational_database_settings: RelationalDatabaseSettings = (
            ConfigurationProvider._load_relational_database_settings(configuration_source_provider)
        )
        self._document_database_settings: DocumentDatabaseSettings = (
            ConfigurationProvider._load_document_database_settings(configuration_source_provider)
        )
        self._json_placeholder_settings: JsonPlaceholderSettings = (
            ConfigurationProvider._load_json_placeholder_settings(configuration_source_provider)
        )

    def get_example_settings(self) -> ExampleSettings:
        return self._example_settings

    def get_relational_database_settings(self) -> RelationalDatabaseSettings:
        return self._relational_database_settings

    def get_document_database_settings(self) -> DocumentDatabaseSettings:
        return self._document_database_settings

    def get_json_placeholder_settings(self) -> JsonPlaceholderSettings:
        return self._json_placeholder_settings

    # Each loader maps a config only after its validator passed it, so each ValidatedValue.expect states a checked
    # fact: it cannot fail unless the validator is wrong.
    @staticmethod
    def _load_example_settings(source_provider: ConfigurationSourceProviderProtocol) -> ExampleSettings:
        config: ExampleConfig = source_provider.get_example_config()
        ValidatorExample.validate_example_config(config)
        return ExampleSettings(greeting_prefix=ValidatedValue.expect(config.greeting_prefix, "ValidatorExample"))

    @staticmethod
    def _load_relational_database_settings(
        source_provider: ConfigurationSourceProviderProtocol,
    ) -> RelationalDatabaseSettings:
        config: RelationalDatabaseConfig = source_provider.get_relational_database_config()
        ValidatorRelationalDatabase.validate_relational_database_config(config)
        return RelationalDatabaseSettings(
            connection_string=ValidatedValue.expect(config.connection_string, "ValidatorRelationalDatabase"),
        )

    @staticmethod
    def _load_document_database_settings(
        source_provider: ConfigurationSourceProviderProtocol,
    ) -> DocumentDatabaseSettings:
        config: DocumentDatabaseConfig = source_provider.get_document_database_config()
        ValidatorDocumentDatabase.validate_document_database_config(config)
        return DocumentDatabaseSettings(
            connection_string=ValidatedValue.expect(config.connection_string, "ValidatorDocumentDatabase"),
        )

    @staticmethod
    def _load_json_placeholder_settings(
        source_provider: ConfigurationSourceProviderProtocol,
    ) -> JsonPlaceholderSettings:
        config: JsonPlaceholderConfig = source_provider.get_json_placeholder_config()
        ValidatorJsonPlaceholder.validate_json_placeholder_config(config)
        return JsonPlaceholderSettings(
            base_url=ValidatedValue.expect(config.base_url, "ValidatorJsonPlaceholder"),
            timeout_seconds=ValidatedValue.expect(config.timeout_seconds, "ValidatorJsonPlaceholder"),
        )
