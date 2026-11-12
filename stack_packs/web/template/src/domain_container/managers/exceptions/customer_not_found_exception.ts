import { RelationalDatabaseGatewayException } from "#src/domain_container/managers/exceptions/relational_database_gateway_exception.ts";

/** No customer has the id asked for: the caller can put it right, so its message is written for the person using the app. */
export class CustomerNotFoundException extends RelationalDatabaseGatewayException {}
