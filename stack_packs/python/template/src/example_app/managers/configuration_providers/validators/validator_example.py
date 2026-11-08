from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.configuration_providers.config_models.example_config import ExampleConfig
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.missing_configuration_exception import MissingConfigurationException


@final
class ValidatorExample:
    @staticmethod
    def validate_example_config(config: ExampleConfig) -> None:
        errors: list[str] = [
            error
            for error in [ValidatorString.validate("greeting_prefix", config.greeting_prefix)]
            if error is not None
        ]

        if errors:
            raise MissingConfigurationException(
                "Example configuration is invalid",
                ContextualData().with_value("setting_name", "example").with_value("errors", "; ".join(errors)),
            )
