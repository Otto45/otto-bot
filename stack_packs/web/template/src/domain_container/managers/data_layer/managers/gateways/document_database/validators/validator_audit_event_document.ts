import { ValidatorString } from "#src/domain_container/managers/common/validators/validator_string.ts";
import type { AuditEventDocument } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/resource_models/audit_event_document.ts";
import { ContextualData } from "#src/domain_container/managers/exceptions/contextual_data.ts";
import { InvalidAuditEventDocumentException } from "#src/domain_container/managers/exceptions/invalid_audit_event_document_exception.ts";

export class ValidatorAuditEventDocument {
  public static validateAuditEventDocument(auditEventDocument: AuditEventDocument): void {
    const errors: string[] = [
      ValidatorString.validate("event_id", auditEventDocument.event_id),
      ValidatorString.validate("customer_id", auditEventDocument.customer_id),
      ValidatorString.validate("description", auditEventDocument.description),
    ].filter((error: string | undefined): error is string => error !== undefined);

    if (auditEventDocument.occurred_on === null || auditEventDocument.occurred_on === undefined) {
      errors.push("occurred_on is missing");
    }

    if (errors.length > 0) {
      throw new InvalidAuditEventDocumentException(
        "Audit event document from the document database audit_events collection failed validation",
        new ContextualData().withValue("resource_name", "audit_event").withValue("errors", errors.join("; ")),
      );
    }
  }
}
