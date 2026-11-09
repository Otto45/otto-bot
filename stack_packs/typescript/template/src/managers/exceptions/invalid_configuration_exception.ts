import { ConfigurationProviderException } from "#src/managers/exceptions/configuration_provider_exception.ts";

/** A setting has a value, but not one the app can use. */
export class InvalidConfigurationException extends ConfigurationProviderException {}
