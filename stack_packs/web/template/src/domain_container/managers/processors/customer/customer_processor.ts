import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidCustomerException } from "#src/domain_container/managers/exceptions/invalid_customer_exception.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";

export class CustomerProcessor {
  /**
   * Throws InvalidCustomerException when the customer can't be saved as given. Its message is written for the person
   * using the app, because a service answers a client error with it.
   */
  public checkCustomer(customer: Customer): void {
    const context: ContextualData = new ContextualData().withValue("customer_id", customer.id);

    if (customer.name.trim() === "") {
      throw new InvalidCustomerException("The customer's name is required", context);
    }
    if (!customer.email.includes("@")) {
      throw new InvalidCustomerException("The customer's email address is not valid", context);
    }
  }
}
