import { RelationalDatabaseGatewayException } from "#src/domain_container/managers/exceptions/relational_database_gateway_exception.ts";

/** A statement sent to Postgres failed, or Postgres could not be reached. */
export class PostgresQueryException extends RelationalDatabaseGatewayException {}
