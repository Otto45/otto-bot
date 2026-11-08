import uuid
from datetime import datetime
from typing import final

from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.user_profile import UserProfile


# Belongs to the CustomerOnboardingEngine: only that engine uses it, so it lives in the engine's folder.
@final
class OnboardingProcessor:
    def build_customer(self, user_profile: UserProfile) -> Customer:
        return Customer(id=f"user-{user_profile.id}", name=user_profile.name, email=user_profile.email)

    def build_onboarded_event(self, customer: Customer, user_profile: UserProfile, occurred_on: datetime) -> AuditEvent:
        return AuditEvent(
            id=uuid.uuid4().hex,
            customer_id=customer.id,
            description=f"Customer was onboarded from user profile {user_profile.username}",
            occurred_on=occurred_on,
        )
