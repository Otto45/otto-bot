import { ValidatorString } from "#src/domain_container/managers/common/validators/validator_string.ts";
import type { CustomerRow } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/resource_models/customer_row.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidPostgresCustomerRowException } from "#src/domain_container/managers/exceptions/invalid_postgres_customer_row_exception.ts";

export class ValidatorCustomerRow {
  public static validateCustomerRow(customerRow: CustomerRow): void {
    const errors: string[] = [
      ValidatorString.validate("customer_id", customerRow.customer_id),
      ValidatorString.validate("full_name", customerRow.full_name),
      ValidatorString.validate("email_address", customerRow.email_address),
    ].filter((error: string | undefined): error is string => error !== undefined);

    if (errors.length > 0) {
      throw new InvalidPostgresCustomerRowException(
        "Customer row from the Postgres customers table failed validation",
        new ContextualData().withValue("resource_name", "customer").withValue("errors", errors.join("; ")),
      );
    }
  }
}
