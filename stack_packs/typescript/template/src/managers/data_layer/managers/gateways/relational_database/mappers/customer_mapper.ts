import { ValidatedValue } from "#src/managers/common/validators/validated_value.ts";
import type { CustomerRow } from "#src/managers/data_layer/managers/gateways/relational_database/resource_models/customer_row.ts";
import type { Customer } from "#src/managers/models/customer.ts";

const VALIDATOR_NAME: string = "ValidatorCustomerRow";

export class CustomerMapper {
  /**
   * Expects a row that has already passed ValidatorCustomerRow: each ValidatedValue.expect states a checked fact, and
   * fails only if the validator is wrong.
   */
  public static toCustomer(customerRow: CustomerRow): Customer {
    return {
      id: ValidatedValue.expect(customerRow.customer_id, VALIDATOR_NAME),
      name: ValidatedValue.expect(customerRow.full_name, VALIDATOR_NAME),
      email: ValidatedValue.expect(customerRow.email_address, VALIDATOR_NAME),
    };
  }
}
