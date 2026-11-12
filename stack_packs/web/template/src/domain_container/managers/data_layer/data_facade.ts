import { DataManager } from "#src/domain_container/managers/data_layer/managers/data_manager.ts";
import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { ServiceLocator } from "#src/domain_container/managers/service_locator/service_locator.ts";

// The data layer's only entry point a manager sees.
export class DataFacade {
  private readonly dataManager: DataManager;

  public constructor(serviceLocator: ServiceLocator) {
    this.dataManager = new DataManager(serviceLocator);
  }

  public async getCustomer(customerId: string): Promise<Customer> {
    return await this.dataManager.getCustomer(customerId);
  }

  public async saveCustomer(customer: Customer): Promise<void> {
    await this.dataManager.saveCustomer(customer);
  }

  public async getAuditEvents(customerId: string): Promise<AuditEvent[]> {
    return await this.dataManager.getAuditEvents(customerId);
  }

  public async saveAuditEvent(auditEvent: AuditEvent): Promise<void> {
    await this.dataManager.saveAuditEvent(auditEvent);
  }

  public close(): void {
    this.dataManager.close();
  }
}
