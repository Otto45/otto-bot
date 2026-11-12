import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { CustomerResponse } from "#src/wire_models/customer_response.ts";

export class CustomerResponseMapper {
  public static toCustomerResponse(customer: Customer): CustomerResponse {
    return { id: customer.id, name: customer.name, email: customer.email };
  }
}
