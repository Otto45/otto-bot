from typing import Final, final

from example_app.managers.data_layer.managers.gateways.document_database.resource_models.document_find_query import (
    DocumentFindQuery,
)
from example_app.managers.data_layer.managers.gateways.document_database.resource_models.document_insert_query import (
    DocumentInsertQuery,
)
from example_app.managers.models.audit_event import AuditEvent

_AUDIT_EVENTS_COLLECTION_NAME: Final[str] = "audit_events"


@final
class AuditEventQueryBuilder:
    @staticmethod
    def build_get_audit_events_query(customer_id: str) -> DocumentFindQuery:
        return DocumentFindQuery(
            collection_name=_AUDIT_EVENTS_COLLECTION_NAME,
            filter_criteria={"customer_id": customer_id},
            sort_field="occurred_on",
        )

    @staticmethod
    def build_save_audit_event_query(audit_event: AuditEvent) -> DocumentInsertQuery:
        return DocumentInsertQuery(
            collection_name=_AUDIT_EVENTS_COLLECTION_NAME,
            document={
                "event_id": audit_event.id,
                "customer_id": audit_event.customer_id,
                "description": audit_event.description,
                # Always with microseconds, so the stored times sort as text in the order they happened.
                "occurred_on": audit_event.occurred_on.isoformat(timespec="microseconds"),
            },
        )
