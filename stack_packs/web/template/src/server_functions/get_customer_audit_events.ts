import { createServerFn } from "@tanstack/react-start";

import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import { AuditEventResponseMapper } from "#src/mappers/audit_event_response_mapper.ts";
import type { AuditEventListResponse } from "#src/wire_models/audit_event_list_response.ts";
import { CustomerRequestSchema } from "#src/wire_models/customer_request.ts";

// A read, so a GET, which a page's loader may call.
export const getCustomerAuditEvents = createServerFn({ method: "GET" })
  .validator(CustomerRequestSchema)
  .handler(async ({ data, context }): Promise<AuditEventListResponse> => {
    const auditEvents: AuditEvent[] = await context.exampleAppFacade.getCustomerAuditEvents(data.customerId);
    return AuditEventResponseMapper.toAuditEventListResponse(auditEvents);
  });
