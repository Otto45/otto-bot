from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.audit_event_document import (
    AuditEventDocument,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_audit_event_document_exception import InvalidAuditEventDocumentException


@final
class ValidatorAuditEventDocument:
    @staticmethod
    def validate_audit_event_document(audit_event_document: AuditEventDocument) -> None:
        errors: list[str] = [
            error
            for error in (
                ValidatorString.validate("event_id", audit_event_document.event_id),
                ValidatorString.validate("customer_id", audit_event_document.customer_id),
                ValidatorString.validate("description", audit_event_document.description),
            )
            if error is not None
        ]

        if audit_event_document.occurred_on is None:
            errors.append("occurred_on is missing")

        if errors:
            raise InvalidAuditEventDocumentException(
                "Audit event document from the document database audit_events collection failed validation",
                ContextualData().with_value("resource_name", "audit_event").with_value("errors", "; ".join(errors)),
            )
