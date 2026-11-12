import { DataFacade } from "#src/domain_container/managers/data_layer/data_facade.ts";
import { ExampleAppManager } from "#src/domain_container/managers/example_app_manager.ts";
import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { Greeting } from "#src/domain_container/managers/models/greeting.ts";
import type { UserProfile } from "#src/domain_container/managers/models/user_profile.ts";
import { ServiceLocator } from "#src/domain_container/managers/service_locator/service_locator.ts";

// The Domain Facade: the domain container's public entry point. Whoever creates one closes it (and everything the
// locator cached) on every path: the server entry (src/server.ts) when the server stops, and anything else that creates
// one holds it with `await using`.
export class ExampleAppFacade implements AsyncDisposable {
  private readonly serviceLocator: ServiceLocator;
  private readonly dataFacade: DataFacade;
  private readonly exampleAppManager: ExampleAppManager;

  /**
   * Creates every dependency, so invalid configuration fails here, before anything opens: the configuration is the
   * first thing anything asks the locator for.
   */
  public constructor(serviceLocator: ServiceLocator = new ServiceLocator()) {
    this.serviceLocator = serviceLocator;
    this.dataFacade = new DataFacade(this.serviceLocator);
    this.exampleAppManager = new ExampleAppManager(this.serviceLocator, this.dataFacade);
  }

  public async [Symbol.asyncDispose](): Promise<void> {
    await this.close();
  }

  public async saveCustomer(customer: Customer): Promise<void> {
    await this.exampleAppManager.saveCustomer(customer);
  }

  public async getCustomerGreeting(customerId: string): Promise<Greeting> {
    return await this.exampleAppManager.getCustomerGreeting(customerId);
  }

  public async getCustomerAuditEvents(customerId: string): Promise<AuditEvent[]> {
    return await this.exampleAppManager.getCustomerAuditEvents(customerId);
  }

  public async getUserProfile(userId: number): Promise<UserProfile> {
    return await this.exampleAppManager.getUserProfile(userId);
  }

  public async onboardCustomer(userId: number): Promise<Customer> {
    return await this.exampleAppManager.onboardCustomer(userId);
  }

  public async close(): Promise<void> {
    this.exampleAppManager.close();
    this.dataFacade.close();
    // Last: the locator closes the singletons (gateways) the managers and the DataFacade were using.
    await this.serviceLocator.close();
  }
}
