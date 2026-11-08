from typing import final

from example_app.managers.data_layer.managers.gateways.document_database.document_database_gateway import (
    DocumentDatabaseGateway,
)
from example_app.managers.data_layer.managers.gateways.relational_database.relational_database_gateway import (
    RelationalDatabaseGateway,
)
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.service_locator.service_locator import ServiceLocator


# Orchestrates every data gateway: a data operation can grow to read from several databases and combine the results
# into one domain model without the layers above changing, and callers never learn which database served a request.
@final
class DataManager:
    def __init__(self, service_locator: ServiceLocator) -> None:
        self._relational_database_gateway: RelationalDatabaseGateway = service_locator.get_relational_database_gateway()
        self._document_database_gateway: DocumentDatabaseGateway = service_locator.get_document_database_gateway()

    async def get_customer(self, customer_id: str) -> Customer:
        return await self._relational_database_gateway.get_customer(customer_id)

    async def save_customer(self, customer: Customer) -> None:
        await self._relational_database_gateway.save_customer(customer)

    async def get_audit_events(self, customer_id: str) -> list[AuditEvent]:
        return await self._document_database_gateway.get_audit_events(customer_id)

    async def save_audit_event(self, audit_event: AuditEvent) -> None:
        await self._document_database_gateway.save_audit_event(audit_event)

    def close(self) -> None:
        """Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
        ServiceLocator closes when the Domain Facade closes it. It becomes async when it owns something whose release
        does I/O."""
