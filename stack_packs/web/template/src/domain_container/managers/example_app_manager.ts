import { randomUUID } from "node:crypto";

import type { DataFacade } from "#src/domain_container/managers/data_layer/data_facade.ts";
import type { CustomerOnboardingEngine } from "#src/domain_container/managers/engines/customer_onboarding/customer_onboarding_engine.ts";
import type { JsonPlaceholderGateway } from "#src/domain_container/managers/gateways/json_placeholder/json_placeholder_gateway.ts";
import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { Greeting } from "#src/domain_container/managers/models/greeting.ts";
import type { UserProfile } from "#src/domain_container/managers/models/user_profile.ts";
import type { CustomerProcessor } from "#src/domain_container/managers/processors/customer/customer_processor.ts";
import type { GreetingProcessor } from "#src/domain_container/managers/processors/greeting/greeting_processor.ts";
import type { ServiceLocator } from "#src/domain_container/managers/service_locator/service_locator.ts";

export class ExampleAppManager {
  private readonly greetingProcessor: GreetingProcessor;
  private readonly customerProcessor: CustomerProcessor;
  // Shared with the Domain Facade, which owns it and closes it.
  private readonly dataFacade: DataFacade;
  private readonly jsonPlaceholderGateway: JsonPlaceholderGateway;
  private readonly customerOnboardingEngine: CustomerOnboardingEngine;

  public constructor(serviceLocator: ServiceLocator, dataFacade: DataFacade) {
    this.greetingProcessor = serviceLocator.getGreetingProcessor();
    this.customerProcessor = serviceLocator.getCustomerProcessor();
    this.dataFacade = dataFacade;
    this.jsonPlaceholderGateway = serviceLocator.getJsonPlaceholderGateway();
    this.customerOnboardingEngine = serviceLocator.getCustomerOnboardingEngine();
  }

  public async saveCustomer(customer: Customer): Promise<void> {
    this.customerProcessor.checkCustomer(customer);

    await this.dataFacade.saveCustomer(customer);
  }

  public async getCustomerGreeting(customerId: string): Promise<Greeting> {
    const customer: Customer = await this.dataFacade.getCustomer(customerId);
    const greeting: Greeting = this.greetingProcessor.greet(customer.name);

    await this.dataFacade.saveAuditEvent({
      id: randomUUID().replaceAll("-", ""),
      customerId: customer.id,
      description: "Customer was greeted",
      occurredOn: new Date(),
    });
    return greeting;
  }

  public async getCustomerAuditEvents(customerId: string): Promise<AuditEvent[]> {
    return await this.dataFacade.getAuditEvents(customerId);
  }

  public async getUserProfile(userId: number): Promise<UserProfile> {
    return await this.jsonPlaceholderGateway.getUserProfile(userId);
  }

  public async onboardCustomer(userId: number): Promise<Customer> {
    return await this.customerOnboardingEngine.onboardCustomer(userId, this.dataFacade);
  }

  /**
   * Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
   * ServiceLocator closes when the Domain Facade closes it. It becomes async when it owns something whose release does
   * I/O.
   */
  public close(): void {}
}
