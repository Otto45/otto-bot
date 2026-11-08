from typing import final

from example_app.managers.exceptions.configuration_provider_exception import ConfigurationProviderException


@final
class InvalidConfigurationException(ConfigurationProviderException):
    """A setting has a value, but not one the app can use."""
