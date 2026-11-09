import { ValidatedValue } from "#src/managers/common/validators/validated_value.ts";
import type { AuditEventDocument } from "#src/managers/data_layer/managers/gateways/document_database/resource_models/audit_event_document.ts";
import type { AuditEvent } from "#src/managers/models/audit_event.ts";

const VALIDATOR_NAME: string = "ValidatorAuditEventDocument";

export class AuditEventMapper {
  /**
   * Expects a document that has already passed ValidatorAuditEventDocument: each ValidatedValue.expect states a
   * checked fact, and fails only if the validator is wrong.
   */
  public static toAuditEvent(auditEventDocument: AuditEventDocument): AuditEvent {
    return {
      id: ValidatedValue.expect(auditEventDocument.event_id, VALIDATOR_NAME),
      customerId: ValidatedValue.expect(auditEventDocument.customer_id, VALIDATOR_NAME),
      description: ValidatedValue.expect(auditEventDocument.description, VALIDATOR_NAME),
      occurredOn: new Date(ValidatedValue.expect(auditEventDocument.occurred_on, VALIDATOR_NAME)),
    };
  }
}
