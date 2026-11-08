from types import TracebackType
from typing import Self, final

from example_app.managers.data_layer.data_facade import DataFacade
from example_app.managers.example_app_manager import ExampleAppManager
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.greeting import Greeting
from example_app.managers.models.user_profile import UserProfile
from example_app.managers.service_locator.service_locator import ServiceLocator


# The Domain Facade: the domain container's public entry point. Whoever creates one uses it in `async with`, which
# closes it (and everything the locator cached) on every path, whether using it failed or not.
@final
class ExampleAppFacade:
    def __init__(self, service_locator: ServiceLocator | None = None) -> None:
        """Creates every dependency, so invalid configuration fails here, before anything opens: the configuration is
        the first thing anything asks the locator for. It is created inside the running event loop, which the database
        pool binds to."""
        self._service_locator: ServiceLocator = service_locator if service_locator is not None else ServiceLocator()
        self._data_facade: DataFacade = DataFacade(self._service_locator)
        self._example_app_manager: ExampleAppManager = ExampleAppManager(self._service_locator, self._data_facade)

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(
        self,
        exception_type: type[BaseException] | None,
        exception: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        await self.close()

    async def save_customer(self, customer: Customer) -> None:
        await self._example_app_manager.save_customer(customer)

    async def get_customer_greeting(self, customer_id: str) -> Greeting:
        return await self._example_app_manager.get_customer_greeting(customer_id)

    async def get_customer_audit_events(self, customer_id: str) -> list[AuditEvent]:
        return await self._example_app_manager.get_customer_audit_events(customer_id)

    async def get_user_profile(self, user_id: int) -> UserProfile:
        return await self._example_app_manager.get_user_profile(user_id)

    async def onboard_customer(self, user_id: int) -> Customer:
        return await self._example_app_manager.onboard_customer(user_id)

    async def close(self) -> None:
        self._example_app_manager.close()
        self._data_facade.close()
        # Last: the locator closes the singletons (gateways) the managers and the DataFacade were using.
        await self._service_locator.close()
