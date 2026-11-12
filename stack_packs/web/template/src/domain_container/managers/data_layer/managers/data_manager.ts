import type { DocumentDatabaseGateway } from "#src/domain_container/managers/data_layer/managers/gateways/document_database/document_database_gateway.ts";
import type { RelationalDatabaseGateway } from "#src/domain_container/managers/data_layer/managers/gateways/relational_database/relational_database_gateway.ts";
import type { AuditEvent } from "#src/domain_container/managers/models/audit_event.ts";
import type { Customer } from "#src/domain_container/managers/models/customer.ts";
import type { ServiceLocator } from "#src/domain_container/managers/service_locator/service_locator.ts";

// Orchestrates every data gateway: a data operation can grow to read from several databases and combine the results into
// one domain model without the layers above changing, and callers never learn which database served a request.
export class DataManager {
  private readonly relationalDatabaseGateway: RelationalDatabaseGateway;
  private readonly documentDatabaseGateway: DocumentDatabaseGateway;

  public constructor(serviceLocator: ServiceLocator) {
    this.relationalDatabaseGateway = serviceLocator.getRelationalDatabaseGateway();
    this.documentDatabaseGateway = serviceLocator.getDocumentDatabaseGateway();
  }

  public async getCustomer(customerId: string): Promise<Customer> {
    return await this.relationalDatabaseGateway.getCustomer(customerId);
  }

  public async saveCustomer(customer: Customer): Promise<void> {
    await this.relationalDatabaseGateway.saveCustomer(customer);
  }

  public async getAuditEvents(customerId: string): Promise<AuditEvent[]> {
    return await this.documentDatabaseGateway.getAuditEvents(customerId);
  }

  public async saveAuditEvent(auditEvent: AuditEvent): Promise<void> {
    await this.documentDatabaseGateway.saveAuditEvent(auditEvent);
  }

  /**
   * Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
   * ServiceLocator closes when the Domain Facade closes it. It becomes async when it owns something whose release does
   * I/O.
   */
  public close(): void {}
}
