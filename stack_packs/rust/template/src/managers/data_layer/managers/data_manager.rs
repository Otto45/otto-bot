use std::sync::Arc;

use crate::managers::data_layer::managers::gateways::document_database::document_database_gateway::DocumentDatabaseGateway;
use crate::managers::data_layer::managers::gateways::relational_database::relational_database_gateway::RelationalDatabaseGateway;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::service_locator::service_locator::ServiceLocator;

pub struct DataManager {
    relational_database_gateway: Arc<RelationalDatabaseGateway>,
    document_database_gateway: Arc<DocumentDatabaseGateway>,
}

impl DataManager {
    pub fn new(service_locator: &ServiceLocator) -> ApplicationResult<Self> {
        Ok(DataManager {
            relational_database_gateway: service_locator.get_relational_database_gateway()?,
            document_database_gateway: service_locator.get_document_database_gateway()?,
        })
    }

    pub async fn get_customer(&self, customer_id: &str) -> ApplicationResult<Customer> {
        self.relational_database_gateway.get_customer(customer_id).await
    }

    pub async fn save_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        self.relational_database_gateway.save_customer(customer).await
    }

    pub async fn get_audit_events(&self, customer_id: &str) -> ApplicationResult<Vec<AuditEvent>> {
        self.document_database_gateway.get_audit_events(customer_id).await
    }

    pub async fn save_audit_event(&self, audit_event: &AuditEvent) -> ApplicationResult<()> {
        self.document_database_gateway.save_audit_event(audit_event).await
    }

    /// Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
    /// ServiceLocator closes when the Domain Facade closes it. It becomes `async` when it owns something whose
    /// release does I/O.
    pub fn close(&self) {}
}
