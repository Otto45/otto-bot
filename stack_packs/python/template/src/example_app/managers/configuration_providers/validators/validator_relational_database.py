from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.configuration_providers.config_models.relational_database_config import (
    RelationalDatabaseConfig,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.missing_configuration_exception import MissingConfigurationException


@final
class ValidatorRelationalDatabase:
    @staticmethod
    def validate_relational_database_config(config: RelationalDatabaseConfig) -> None:
        errors: list[str] = [
            error
            for error in [ValidatorString.validate("connection_string", config.connection_string)]
            if error is not None
        ]

        if errors:
            raise MissingConfigurationException(
                "Relational database configuration is invalid",
                ContextualData()
                .with_value("setting_name", "relational_database")
                .with_value("errors", "; ".join(errors)),
            )
