import { ConfigurationProviderException } from "#src/managers/exceptions/configuration_provider_exception.ts";

/** A setting the app needs has no value. */
export class MissingConfigurationException extends ConfigurationProviderException {}
