from typing import Final, final

from example_app.managers.common.validators.validated_value import ValidatedValue
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.audit_event_document import (
    AuditEventDocument,
)
from example_app.managers.models.audit_event import AuditEvent

_VALIDATOR_NAME: Final[str] = "ValidatorAuditEventDocument"


@final
class AuditEventMapper:
    @staticmethod
    def to_audit_event(audit_event_document: AuditEventDocument) -> AuditEvent:
        """Expects a document that has already passed ValidatorAuditEventDocument: each ValidatedValue.expect states a
        checked fact, and fails only if the validator is wrong."""
        return AuditEvent(
            id=ValidatedValue.expect(audit_event_document.event_id, _VALIDATOR_NAME),
            customer_id=ValidatedValue.expect(audit_event_document.customer_id, _VALIDATOR_NAME),
            description=ValidatedValue.expect(audit_event_document.description, _VALIDATOR_NAME),
            occurred_on=ValidatedValue.expect(audit_event_document.occurred_on, _VALIDATOR_NAME),
        )
