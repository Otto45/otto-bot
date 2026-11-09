import { randomUUID } from "node:crypto";

import type { AuditEvent } from "#src/managers/models/audit_event.ts";
import type { Customer } from "#src/managers/models/customer.ts";
import type { UserProfile } from "#src/managers/models/user_profile.ts";

// Belongs to the CustomerOnboardingEngine: only that engine uses it, so it lives in the engine's folder.
export class OnboardingProcessor {
  public buildCustomer(userProfile: UserProfile): Customer {
    return { id: `user-${userProfile.id}`, name: userProfile.name, email: userProfile.email };
  }

  public buildOnboardedEvent(customer: Customer, userProfile: UserProfile, occurredOn: Date): AuditEvent {
    return {
      id: randomUUID().replaceAll("-", ""),
      customerId: customer.id,
      description: `Customer was onboarded from user profile ${userProfile.username}`,
      occurredOn,
    };
  }
}
