from typing import final

from example_app.managers.data_layer.managers.data_manager import DataManager
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.service_locator.service_locator import ServiceLocator


# The data layer's only entry point a manager sees.
@final
class DataFacade:
    def __init__(self, service_locator: ServiceLocator) -> None:
        self._data_manager: DataManager = DataManager(service_locator)

    async def get_customer(self, customer_id: str) -> Customer:
        return await self._data_manager.get_customer(customer_id)

    async def save_customer(self, customer: Customer) -> None:
        await self._data_manager.save_customer(customer)

    async def get_audit_events(self, customer_id: str) -> list[AuditEvent]:
        return await self._data_manager.get_audit_events(customer_id)

    async def save_audit_event(self, audit_event: AuditEvent) -> None:
        await self._data_manager.save_audit_event(audit_event)

    def close(self) -> None:
        self._data_manager.close()
