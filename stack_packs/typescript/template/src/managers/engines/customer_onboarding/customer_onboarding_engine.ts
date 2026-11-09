import type { DataFacade } from "#src/managers/data_layer/data_facade.ts";
import type { OnboardingProcessor } from "#src/managers/engines/customer_onboarding/processors/onboarding/onboarding_processor.ts";
import type { JsonPlaceholderGateway } from "#src/managers/gateways/json_placeholder/json_placeholder_gateway.ts";
import type { AuditEvent } from "#src/managers/models/audit_event.ts";
import type { Customer } from "#src/managers/models/customer.ts";
import type { UserProfile } from "#src/managers/models/user_profile.ts";

// Onboarding is one task to the manager, with several kinds of work behind it: an external lookup, deciding what the
// customer looks like, saving it, and recording that it happened. The engine sequences those steps so the manager stays
// a list of one-line coordinations.
export class CustomerOnboardingEngine {
  private readonly jsonPlaceholderGateway: JsonPlaceholderGateway;
  private readonly onboardingProcessor: OnboardingProcessor;

  public constructor(jsonPlaceholderGateway: JsonPlaceholderGateway, onboardingProcessor: OnboardingProcessor) {
    this.jsonPlaceholderGateway = jsonPlaceholderGateway;
    this.onboardingProcessor = onboardingProcessor;
  }

  /**
   * The engine is a shared singleton built by the ServiceLocator, which never sees the DataFacade, so the manager
   * passes its DataFacade in with each call.
   */
  public async onboardCustomer(userId: number, dataFacade: DataFacade): Promise<Customer> {
    const userProfile: UserProfile = await this.jsonPlaceholderGateway.getUserProfile(userId);
    const customer: Customer = this.onboardingProcessor.buildCustomer(userProfile);

    await dataFacade.saveCustomer(customer);
    const onboardedEvent: AuditEvent = this.onboardingProcessor.buildOnboardedEvent(customer, userProfile, new Date());
    await dataFacade.saveAuditEvent(onboardedEvent);
    return customer;
  }
}
