import { createServerFn } from "@tanstack/react-start";

import type { Greeting } from "#src/domain_container/managers/models/greeting.ts";
import { GreetingResponseMapper } from "#src/mappers/greeting_response_mapper.ts";
import { CustomerRequestSchema } from "#src/wire_models/customer_request.ts";
import type { GreetingResponse } from "#src/wire_models/greeting_response.ts";

// Greets a saved customer, which records it in their audit trail: a write, so a POST that a page calls when the person
// asks for it, never from a loader.
export const greetCustomer = createServerFn({ method: "POST" })
  .validator(CustomerRequestSchema)
  .handler(async ({ data, context }): Promise<GreetingResponse> => {
    const greeting: Greeting = await context.exampleAppFacade.getCustomerGreeting(data.customerId);
    return GreetingResponseMapper.toGreetingResponse(greeting);
  });
