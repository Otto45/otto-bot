import type { ConfigurationProviderException } from "#src/managers/exceptions/configuration_provider_exception.ts";
import type { DocumentDatabaseGatewayException } from "#src/managers/exceptions/document_database_gateway_exception.ts";
import type { InvalidCustomerException } from "#src/managers/exceptions/invalid_customer_exception.ts";
import type { JsonPlaceholderGatewayException } from "#src/managers/exceptions/json_placeholder_gateway_exception.ts";
import type { RelationalDatabaseGatewayException } from "#src/managers/exceptions/relational_database_gateway_exception.ts";

// Every family of the application's exceptions. A switch that decides something for every exception (e.g.
// HttpErrorMapper.toStatusCode) switches on `exception.getFamily().family` and names each family here, with no
// `default`, so it fails to compile (and to lint: switch-exhaustiveness-check) until a new family is decided there.
export type ExceptionFamily =
  | ConfigurationProviderException
  | RelationalDatabaseGatewayException
  | DocumentDatabaseGatewayException
  | JsonPlaceholderGatewayException
  | InvalidCustomerException;
