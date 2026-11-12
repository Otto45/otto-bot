import { createServerFn } from "@tanstack/react-start";

import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import { CustomerResponseMapper } from "#src/mappers/customer_response_mapper.ts";
import type { CustomerResponse } from "#src/wire_models/customer_response.ts";
import { UserProfileRequestSchema } from "#src/wire_models/user_profile_request.ts";

export const onboardCustomer = createServerFn({ method: "POST" })
  .validator(UserProfileRequestSchema)
  .handler(async ({ data, context }): Promise<CustomerResponse> => {
    const customer: Customer = await context.exampleAppFacade.onboardCustomer(data.userId);
    return CustomerResponseMapper.toCustomerResponse(customer);
  });
