import { createRoute } from "@hono/zod-openapi";
import type { OpenAPIHono } from "@hono/zod-openapi";

import type { AuditEvent } from "#src/managers/models/audit_event.ts";
import type { Customer } from "#src/managers/models/customer.ts";
import type { Greeting } from "#src/managers/models/greeting.ts";
import type { HttpEnvironment } from "#src/services/http/http_environment.ts";
import { AuditEventResponseMapper } from "#src/services/http/mappers/audit_event_response_mapper.ts";
import { GreetingResponseMapper } from "#src/services/http/mappers/greeting_response_mapper.ts";
import { SaveCustomerRequestMapper } from "#src/services/http/mappers/save_customer_request_mapper.ts";
import { AuditEventListResponseSchema } from "#src/services/http/wire_models/audit_event_list_response.ts";
import { CustomerPathParametersSchema } from "#src/services/http/wire_models/customer_path_parameters.ts";
import type { CustomerPathParameters } from "#src/services/http/wire_models/customer_path_parameters.ts";
import { ErrorResponseSchema } from "#src/services/http/wire_models/error_response.ts";
import { GreetingResponseSchema } from "#src/services/http/wire_models/greeting_response.ts";
import { SaveCustomerRequestSchema } from "#src/services/http/wire_models/save_customer_request.ts";
import type { SaveCustomerRequest } from "#src/services/http/wire_models/save_customer_request.ts";

// Each route maps the request into domain models, calls one facade operation, and maps the domain models it returns to
// a response model. Domain models never cross the wire in either direction. Each route also names the errors it can
// answer with (`responses`), so they are part of the service's contract (openapi.json). The facade comes from the
// request's context, which holds the one facade the service created; nothing builds one.
export class CustomerRouter {
  public static register(application: OpenAPIHono<HttpEnvironment>): void {
    application.openapi(
      createRoute({
        method: "put",
        path: "/customers/{customer_id}",
        request: {
          params: CustomerPathParametersSchema,
          body: { content: { "application/json": { schema: SaveCustomerRequestSchema } }, required: true },
        },
        responses: {
          204: { description: "The customer was saved" },
          400: {
            description: "The customer can't be saved as given",
            content: { "application/json": { schema: ErrorResponseSchema } },
          },
          422: {
            description: "The body does not fit the request model",
            content: { "application/json": { schema: ErrorResponseSchema } },
          },
        },
      }),
      async (context) => {
        const parameters: CustomerPathParameters = context.req.valid("param");
        const saveCustomerRequest: SaveCustomerRequest = context.req.valid("json");
        const customer: Customer = SaveCustomerRequestMapper.toCustomer(parameters.customer_id, saveCustomerRequest);

        await context.get("exampleAppFacade").saveCustomer(customer);
        return context.body(null, 204);
      },
    );

    application.openapi(
      createRoute({
        method: "post",
        path: "/customers/{customer_id}/greetings",
        request: { params: CustomerPathParametersSchema },
        responses: {
          201: {
            description: "The customer was greeted",
            content: { "application/json": { schema: GreetingResponseSchema } },
          },
          404: {
            description: "No customer has this id",
            content: { "application/json": { schema: ErrorResponseSchema } },
          },
        },
      }),
      async (context) => {
        const parameters: CustomerPathParameters = context.req.valid("param");
        const greeting: Greeting = await context.get("exampleAppFacade").getCustomerGreeting(parameters.customer_id);

        return context.json(GreetingResponseMapper.toGreetingResponse(greeting), 201);
      },
    );

    application.openapi(
      createRoute({
        method: "get",
        path: "/customers/{customer_id}/audit-events",
        request: { params: CustomerPathParametersSchema },
        responses: {
          200: {
            description: "The customer's audit events, oldest first",
            content: { "application/json": { schema: AuditEventListResponseSchema } },
          },
        },
      }),
      async (context) => {
        const parameters: CustomerPathParameters = context.req.valid("param");
        const auditEvents: AuditEvent[] = await context
          .get("exampleAppFacade")
          .getCustomerAuditEvents(parameters.customer_id);

        return context.json(AuditEventResponseMapper.toAuditEventListResponse(auditEvents), 200);
      },
    );
  }
}
