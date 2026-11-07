use crate::managers::data_layer::managers::data_manager::DataManager;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::service_locator::service_locator::ServiceLocator;

pub struct DataFacade {
    data_manager: DataManager,
}

impl DataFacade {
    pub fn new(service_locator: &ServiceLocator) -> ApplicationResult<Self> {
        Ok(DataFacade {
            data_manager: DataManager::new(service_locator)?,
        })
    }

    pub async fn get_customer(&self, customer_id: &str) -> ApplicationResult<Customer> {
        self.data_manager.get_customer(customer_id).await
    }

    pub async fn save_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        self.data_manager.save_customer(customer).await
    }

    pub async fn get_audit_events(&self, customer_id: &str) -> ApplicationResult<Vec<AuditEvent>> {
        self.data_manager.get_audit_events(customer_id).await
    }

    pub async fn save_audit_event(&self, audit_event: &AuditEvent) -> ApplicationResult<()> {
        self.data_manager.save_audit_event(audit_event).await
    }

    pub fn close(&self) {
        self.data_manager.close();
    }
}
