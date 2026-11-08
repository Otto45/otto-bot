import uuid
from datetime import UTC, datetime
from typing import final

from example_app.managers.data_layer.data_facade import DataFacade
from example_app.managers.engines.customer_onboarding.customer_onboarding_engine import CustomerOnboardingEngine
from example_app.managers.gateways.json_placeholder.json_placeholder_gateway import JsonPlaceholderGateway
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.greeting import Greeting
from example_app.managers.models.user_profile import UserProfile
from example_app.managers.processors.customer.customer_processor import CustomerProcessor
from example_app.managers.processors.greeting.greeting_processor import GreetingProcessor
from example_app.managers.service_locator.service_locator import ServiceLocator


@final
class ExampleAppManager:
    def __init__(self, service_locator: ServiceLocator, data_facade: DataFacade) -> None:
        self._greeting_processor: GreetingProcessor = service_locator.get_greeting_processor()
        self._customer_processor: CustomerProcessor = service_locator.get_customer_processor()
        # Shared with the Domain Facade, which owns it and closes it.
        self._data_facade: DataFacade = data_facade
        self._json_placeholder_gateway: JsonPlaceholderGateway = service_locator.get_json_placeholder_gateway()
        self._customer_onboarding_engine: CustomerOnboardingEngine = service_locator.get_customer_onboarding_engine()

    async def save_customer(self, customer: Customer) -> None:
        self._customer_processor.check_customer(customer)

        await self._data_facade.save_customer(customer)

    async def get_customer_greeting(self, customer_id: str) -> Greeting:
        customer: Customer = await self._data_facade.get_customer(customer_id)
        greeting: Greeting = self._greeting_processor.greet(customer.name)

        await self._data_facade.save_audit_event(
            AuditEvent(
                id=uuid.uuid4().hex,
                customer_id=customer.id,
                description="Customer was greeted",
                occurred_on=datetime.now(UTC),
            )
        )
        return greeting

    async def get_customer_audit_events(self, customer_id: str) -> list[AuditEvent]:
        return await self._data_facade.get_audit_events(customer_id)

    async def get_user_profile(self, user_id: int) -> UserProfile:
        return await self._json_placeholder_gateway.get_user_profile(user_id)

    async def onboard_customer(self, user_id: int) -> Customer:
        return await self._customer_onboarding_engine.onboard_customer(user_id, self._data_facade)

    def close(self) -> None:
        """Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
        ServiceLocator closes when the Domain Facade closes it. It becomes async when it owns something whose release
        does I/O."""
