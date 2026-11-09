import { SqlQuery } from "#src/managers/data_layer/managers/gateways/relational_database/resource_models/sql_query.ts";
import type { Customer } from "#src/managers/models/customer.ts";

export class CustomerQueryBuilder {
  public static buildGetCustomerQuery(customerId: string): SqlQuery {
    return SqlQuery.of("SELECT customer_id, full_name, email_address FROM customers WHERE customer_id = $1", [
      customerId,
    ]);
  }

  public static buildSaveCustomerQuery(customer: Customer): SqlQuery {
    return SqlQuery.of(
      `INSERT INTO customers (customer_id, full_name, email_address)
      VALUES ($1, $2, $3)
      ON CONFLICT (customer_id) DO UPDATE SET full_name = excluded.full_name, email_address = excluded.email_address`,
      [customer.id, customer.name, customer.email],
    );
  }
}
