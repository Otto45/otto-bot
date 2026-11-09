import { RelationalDatabaseGatewayException } from "#src/managers/exceptions/relational_database_gateway_exception.ts";

/** A customers row doesn't have what a Customer needs. */
export class InvalidPostgresCustomerRowException extends RelationalDatabaseGatewayException {}
