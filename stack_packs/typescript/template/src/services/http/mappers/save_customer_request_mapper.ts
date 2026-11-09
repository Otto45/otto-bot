import type { Customer } from "#src/managers/models/customer.ts";
import type { SaveCustomerRequest } from "#src/services/http/wire_models/save_customer_request.ts";

export class SaveCustomerRequestMapper {
  public static toCustomer(customerId: string, saveCustomerRequest: SaveCustomerRequest): Customer {
    return { id: customerId, name: saveCustomerRequest.name, email: saveCustomerRequest.email };
  }
}
