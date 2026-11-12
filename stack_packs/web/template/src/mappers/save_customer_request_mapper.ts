import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { SaveCustomerRequest } from "#src/wire_models/save_customer_request.ts";

// Maps between wire models and domain models, on the server: domain models never reach the browser, and the browser's
// requests never reach the facade as they came.
export class SaveCustomerRequestMapper {
  public static toCustomer(saveCustomerRequest: SaveCustomerRequest): Customer {
    return { id: saveCustomerRequest.customerId, name: saveCustomerRequest.name, email: saveCustomerRequest.email };
  }
}
