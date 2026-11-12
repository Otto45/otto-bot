import { createServerFn } from "@tanstack/react-start";

import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import { SaveCustomerRequestMapper } from "#src/mappers/save_customer_request_mapper.ts";
import { SaveCustomerRequestSchema } from "#src/wire_models/save_customer_request.ts";

// A server function runs only on the server, and a page calls it as it would any function (the browser's call is a
// request to the server). Each one checks its input against a request model, maps it into domain models, calls one
// facade operation (the one facade, from its context), and maps what that returns to a response model. Its exceptions
// are applicationExceptionMiddleware's to log and tell; it never catches one itself.
export const saveCustomer = createServerFn({ method: "POST" })
  .validator(SaveCustomerRequestSchema)
  .handler(async ({ data, context }): Promise<void> => {
    const customer: Customer = SaveCustomerRequestMapper.toCustomer(data);
    await context.exampleAppFacade.saveCustomer(customer);
  });
