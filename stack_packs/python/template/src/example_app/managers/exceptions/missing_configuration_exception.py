from typing import final

from example_app.managers.exceptions.configuration_provider_exception import ConfigurationProviderException


@final
class MissingConfigurationException(ConfigurationProviderException):
    """A setting the app needs has no value."""
