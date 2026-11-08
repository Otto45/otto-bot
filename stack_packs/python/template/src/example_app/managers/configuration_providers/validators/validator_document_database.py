from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.configuration_providers.config_models.document_database_config import DocumentDatabaseConfig
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.missing_configuration_exception import MissingConfigurationException


@final
class ValidatorDocumentDatabase:
    @staticmethod
    def validate_document_database_config(config: DocumentDatabaseConfig) -> None:
        errors: list[str] = [
            error
            for error in [ValidatorString.validate("connection_string", config.connection_string)]
            if error is not None
        ]

        if errors:
            raise MissingConfigurationException(
                "Document database configuration is invalid",
                ContextualData()
                .with_value("setting_name", "document_database")
                .with_value("errors", "; ".join(errors)),
            )
