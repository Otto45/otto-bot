import type { DocumentFindQuery } from "#src/managers/data_layer/managers/gateways/document_database/resource_models/document_find_query.ts";
import type { DocumentInsertQuery } from "#src/managers/data_layer/managers/gateways/document_database/resource_models/document_insert_query.ts";
import type { AuditEvent } from "#src/managers/models/audit_event.ts";

const AUDIT_EVENTS_COLLECTION_NAME: string = "audit_events";

export class AuditEventQueryBuilder {
  public static buildGetAuditEventsQuery(customerId: string): DocumentFindQuery {
    return {
      collectionName: AUDIT_EVENTS_COLLECTION_NAME,
      filterCriteria: { customer_id: customerId },
      sortField: "occurred_on",
    };
  }

  public static buildSaveAuditEventQuery(auditEvent: AuditEvent): DocumentInsertQuery {
    return {
      collectionName: AUDIT_EVENTS_COLLECTION_NAME,
      document: {
        event_id: auditEvent.id,
        customer_id: auditEvent.customerId,
        description: auditEvent.description,
        // Always to the millisecond, in UTC, so the stored times sort as text in the order they happened.
        occurred_on: auditEvent.occurredOn.toISOString(),
      },
    };
  }
}
