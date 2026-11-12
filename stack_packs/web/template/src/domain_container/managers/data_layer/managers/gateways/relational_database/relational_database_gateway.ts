import { inspect } from "node:util";

import pg from "pg";
import type { z } from "zod";

import { ValidationErrorFormatter } from "#src/domain_container/managers/common/formatters/validation_error_formatter.ts";
import { CustomerMapper } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/mappers/customer_mapper.ts";
import { CustomerQueryBuilder } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/query_builders/customer_query_builder.ts";
import { CustomerRowSchema } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/resource_models/customer_row.ts";
import type { CustomerRow } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/resource_models/customer_row.ts";
import type { SqlQuery } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/resource_models/sql_query.ts";
import { ValidatorCustomerRow } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/validators/validator_customer_row.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { CustomerNotFoundException } from "#src/domain_container/managers/exceptions/customer_not_found_exception.ts";
import { InvalidPostgresCustomerRowException } from "#src/domain_container/managers/exceptions/invalid_postgres_customer_row_exception.ts";
import { PostgresQueryException } from "#src/domain_container/managers/exceptions/postgres_query_exception.ts";
import { ApplicationLogger } from "#src/domain_container/managers/logger/application_logger.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";

export class RelationalDatabaseGateway {
  private readonly pool: pg.Pool;

  /**
   * Given a pool the seam built, which connects only when a statement is sent, so building it opens nothing. The schema
   * is not the gateway's: it is migrated outside the application (migrations/, applied by dbmate).
   */
  public constructor(pool: pg.Pool) {
    this.pool = pool;
    // An idle connection the server dropped is reported here; unhandled, it would end the process. The next statement
    // gets a new connection.
    this.pool.on("error", (error: Error): void => {
      ApplicationLogger.warning("An idle Postgres connection failed", new ContextualData(), error);
    });
  }

  public async getCustomer(customerId: string): Promise<Customer> {
    const context: ContextualData = new ContextualData().withValue("customer_id", customerId);
    const sqlQuery: SqlQuery = CustomerQueryBuilder.buildGetCustomerQuery(customerId);

    const rows: unknown[] = await this.query(sqlQuery, "retrieving a customer", context);
    const row: unknown = rows[0];
    if (row === undefined) {
      throw new CustomerNotFoundException("The customer was not found", context);
    }

    const customerRow: CustomerRow = RelationalDatabaseGateway.toCustomerRow(row, context);
    ValidatorCustomerRow.validateCustomerRow(customerRow);
    return CustomerMapper.toCustomer(customerRow);
  }

  public async saveCustomer(customer: Customer): Promise<void> {
    const context: ContextualData = new ContextualData().withValue("customer_id", customer.id);
    const sqlQuery: SqlQuery = CustomerQueryBuilder.buildSaveCustomerQuery(customer);

    await this.query(sqlQuery, "saving a customer", context);
  }

  /** Closes the pool's connections. A pool that never connected has none. */
  public async close(): Promise<void> {
    await this.pool.end();
  }

  private async query(sqlQuery: SqlQuery, operationDescription: string, context: ContextualData): Promise<unknown[]> {
    try {
      const result: pg.QueryResult = await this.pool.query(sqlQuery.statement, [...sqlQuery.parameters]);
      const rows: unknown[] = result.rows;
      return rows;
    } catch (error: unknown) {
      throw RelationalDatabaseGateway.toQueryException(error, sqlQuery, operationDescription, context);
    }
  }

  private static toCustomerRow(row: unknown, context: ContextualData): CustomerRow {
    const parsed: z.ZodSafeParseResult<CustomerRow> = CustomerRowSchema.safeParse(row);
    if (!parsed.success) {
      throw new InvalidPostgresCustomerRowException(
        "Customer row from the Postgres customers table could not be deserialized",
        context.withValue("errors", ValidationErrorFormatter.format(parsed.error, "row")),
      );
    }
    return parsed.data;
  }

  // A refused connection is an AggregateError (one error per address tried), whose own message is empty.
  private static describeError(error: unknown): string {
    if (error instanceof AggregateError) {
      return error.errors
        .map((innerError: unknown): string => (innerError instanceof Error ? innerError.message : inspect(innerError)))
        .join("; ");
    }
    return error instanceof Error ? error.message : inspect(error);
  }

  private static toQueryException(
    error: unknown,
    sqlQuery: SqlQuery,
    operationDescription: string,
    context: ContextualData,
  ): PostgresQueryException {
    // Parameter values stay out of the context: they can hold customer data.
    let exceptionContext: ContextualData = context
      .withValue("error_type", error instanceof Error ? error.constructor.name : typeof error)
      .withValue("postgres_error_message", RelationalDatabaseGateway.describeError(error))
      .withValue("sql_statement", sqlQuery.statement);
    // Only set when the error comes from Postgres itself (e.g. not for a refused connection): its SQLSTATE code.
    if (error instanceof pg.DatabaseError && error.code !== undefined) {
      exceptionContext = exceptionContext.withValue("postgres_error_code", error.code);
    }
    return new PostgresQueryException(`Postgres query failed while ${operationDescription}`, exceptionContext, {
      cause: error,
    });
  }
}
