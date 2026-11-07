use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::customer_onboarding::customer_onboarding_engine::CustomerOnboardingEngine;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::json_placeholder::json_placeholder_gateway::JsonPlaceholderGateway;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::models::greeting::Greeting;
use crate::managers::models::user_profile::UserProfile;
use crate::managers::processors::customer::customer_processor::CustomerProcessor;
use crate::managers::processors::greeting::greeting_processor::GreetingProcessor;
use crate::managers::service_locator::service_locator::ServiceLocator;

pub struct ExampleAppManager {
    greeting_processor: Arc<GreetingProcessor>,
    customer_processor: Arc<CustomerProcessor>,
    // Shared with the Domain Facade, which owns it and closes it.
    data_facade: Arc<DataFacade>,
    json_placeholder_gateway: Arc<JsonPlaceholderGateway>,
    customer_onboarding_engine: Arc<CustomerOnboardingEngine>,
}

impl ExampleAppManager {
    pub fn new(service_locator: &ServiceLocator, data_facade: Arc<DataFacade>) -> ApplicationResult<Self> {
        Ok(ExampleAppManager {
            greeting_processor: service_locator.get_greeting_processor()?,
            customer_processor: service_locator.get_customer_processor()?,
            data_facade,
            json_placeholder_gateway: service_locator.get_json_placeholder_gateway()?,
            customer_onboarding_engine: service_locator.get_customer_onboarding_engine()?,
        })
    }

    pub async fn save_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        self.customer_processor.check_customer(customer)?;

        self.data_facade.save_customer(customer).await
    }

    pub async fn get_customer_greeting(&self, customer_id: &str) -> ApplicationResult<Greeting> {
        let customer: Customer = self.data_facade.get_customer(customer_id).await?;
        let greeting: Greeting = self.greeting_processor.greet(&customer.name);

        self.data_facade
            .save_audit_event(&AuditEvent {
                id: Uuid::new_v4().simple().to_string(),
                customer_id: customer.id,
                description: "Customer was greeted".to_string(),
                occurred_on: Utc::now(),
            })
            .await?;

        Ok(greeting)
    }

    pub async fn get_customer_audit_events(&self, customer_id: &str) -> ApplicationResult<Vec<AuditEvent>> {
        self.data_facade.get_audit_events(customer_id).await
    }

    pub async fn get_user_profile(&self, user_id: i64) -> ApplicationResult<UserProfile> {
        self.json_placeholder_gateway.get_user_profile(user_id).await
    }

    pub async fn onboard_customer(&self, user_id: i64) -> ApplicationResult<Customer> {
        self.customer_onboarding_engine
            .onboard_customer(user_id, &self.data_facade)
            .await
    }

    /// Releases what this manager owns itself. It owns nothing yet: the gateways it uses are singletons that the
    /// ServiceLocator closes when the Domain Facade closes it. It becomes `async` when it owns something whose
    /// release does I/O.
    pub fn close(&self) {}
}
