from datetime import UTC, datetime
from typing import TYPE_CHECKING, final

from example_app.managers.engines.customer_onboarding.processors.onboarding.onboarding_processor import (
    OnboardingProcessor,
)
from example_app.managers.gateways.json_placeholder.json_placeholder_gateway import JsonPlaceholderGateway
from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.user_profile import UserProfile

if TYPE_CHECKING:
    # The DataFacade's module imports the ServiceLocator's, which imports this one: the engine only names its type.
    from example_app.managers.data_layer.data_facade import DataFacade


# Onboarding is one task to the manager, with several kinds of work behind it: an external lookup, deciding what the
# customer looks like, saving it, and recording that it happened. The engine sequences those steps so the manager stays
# a list of one-line coordinations.
@final
class CustomerOnboardingEngine:
    def __init__(
        self, json_placeholder_gateway: JsonPlaceholderGateway, onboarding_processor: OnboardingProcessor
    ) -> None:
        self._json_placeholder_gateway: JsonPlaceholderGateway = json_placeholder_gateway
        self._onboarding_processor: OnboardingProcessor = onboarding_processor

    async def onboard_customer(self, user_id: int, data_facade: DataFacade) -> Customer:
        """The engine is a shared singleton built by the ServiceLocator, which never sees the DataFacade, so the
        manager passes its DataFacade in with each call."""
        user_profile: UserProfile = await self._json_placeholder_gateway.get_user_profile(user_id)
        customer: Customer = self._onboarding_processor.build_customer(user_profile)

        await data_facade.save_customer(customer)
        onboarded_event: AuditEvent = self._onboarding_processor.build_onboarded_event(
            customer, user_profile, datetime.now(UTC)
        )
        await data_facade.save_audit_event(onboarded_event)
        return customer
