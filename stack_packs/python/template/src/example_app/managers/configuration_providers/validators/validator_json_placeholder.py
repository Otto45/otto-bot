import math
from typing import Final, final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.configuration_providers.config_models.json_placeholder_config import JsonPlaceholderConfig
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_configuration_exception import InvalidConfigurationException
from example_app.managers.exceptions.missing_configuration_exception import MissingConfigurationException

# More retries than this would hold a request for minutes on an outage that is no longer brief.
_MAX_RETRIES_LIMIT: Final[int] = 10


@final
class ValidatorJsonPlaceholder:
    @staticmethod
    def validate_json_placeholder_config(config: JsonPlaceholderConfig) -> None:
        errors: list[str] = []

        base_url_error: str | None = ValidatorString.validate("base_url", config.base_url)
        if base_url_error is not None:
            errors.append(base_url_error)

        if config.timeout_seconds is None:
            errors.append("timeout_seconds is missing")
        if config.max_retries is None:
            errors.append("max_retries is missing")

        if errors:
            raise MissingConfigurationException(
                "JSON Placeholder configuration is invalid",
                ContextualData().with_value("setting_name", "json_placeholder").with_value("errors", "; ".join(errors)),
            )

        if config.max_retries is not None and not 0 <= config.max_retries <= _MAX_RETRIES_LIMIT:
            raise InvalidConfigurationException(
                "JSON Placeholder configuration is invalid",
                ContextualData()
                .with_value("setting_name", "json_placeholder")
                .with_value("errors", f"max_retries is not between 0 and {_MAX_RETRIES_LIMIT}"),
            )

        timeout_seconds: float | None = config.timeout_seconds
        if timeout_seconds is None:
            return
        # NaN is not positive either; an infinite timeout is no timeout.
        timeout_error: str | None = None
        if math.isnan(timeout_seconds) or timeout_seconds <= 0:
            timeout_error: str | None = "timeout_seconds is not positive"
        elif math.isinf(timeout_seconds):
            timeout_error: str | None = "timeout_seconds is not finite"
        if timeout_error is not None:
            raise InvalidConfigurationException(
                "JSON Placeholder configuration is invalid",
                ContextualData().with_value("setting_name", "json_placeholder").with_value("errors", timeout_error),
            )
