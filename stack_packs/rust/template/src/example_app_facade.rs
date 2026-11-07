use std::sync::Arc;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::example_app_manager::ExampleAppManager;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::models::greeting::Greeting;
use crate::managers::models::user_profile::UserProfile;
use crate::managers::service_locator::service_locator::ServiceLocator;

/// The Domain Facade: the domain container's public entry point.
///
/// Rust has no `async with` (a value can't await anything while it is dropped), so whoever creates a facade calls
/// `close().await` when done with it, on every path, the way main() and the HTTP service do.
pub struct ExampleAppFacade {
    service_locator: Arc<ServiceLocator>,
    data_facade: Arc<DataFacade>,
    example_app_manager: ExampleAppManager,
}

impl ExampleAppFacade {
    /// Creates every dependency, so invalid configuration fails here. The database pool it builds needs a Tokio
    /// runtime, so a facade is created inside one.
    pub fn new() -> ApplicationResult<Self> {
        ExampleAppFacade::with_service_locator(Arc::new(ServiceLocator::new()))
    }

    pub fn with_service_locator(service_locator: Arc<ServiceLocator>) -> ApplicationResult<Self> {
        let data_facade: Arc<DataFacade> = Arc::new(DataFacade::new(&service_locator)?);
        let example_app_manager: ExampleAppManager = ExampleAppManager::new(&service_locator, data_facade.clone())?;

        Ok(ExampleAppFacade {
            service_locator,
            data_facade,
            example_app_manager,
        })
    }

    pub async fn save_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        self.example_app_manager.save_customer(customer).await
    }

    pub async fn get_customer_greeting(&self, customer_id: &str) -> ApplicationResult<Greeting> {
        self.example_app_manager.get_customer_greeting(customer_id).await
    }

    pub async fn get_customer_audit_events(&self, customer_id: &str) -> ApplicationResult<Vec<AuditEvent>> {
        self.example_app_manager.get_customer_audit_events(customer_id).await
    }

    pub async fn get_user_profile(&self, user_id: i64) -> ApplicationResult<UserProfile> {
        self.example_app_manager.get_user_profile(user_id).await
    }

    pub async fn onboard_customer(&self, user_id: i64) -> ApplicationResult<Customer> {
        self.example_app_manager.onboard_customer(user_id).await
    }

    pub async fn close(&self) {
        self.example_app_manager.close();
        self.data_facade.close();
        // Last: the locator closes the singletons (gateways) the managers and the DataFacade were using.
        self.service_locator.close().await;
    }
}
