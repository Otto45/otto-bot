use std::sync::Arc;

use chrono::Utc;

use crate::managers::data_layer::data_facade::DataFacade;
use crate::managers::engines::customer_onboarding::processors::onboarding::onboarding_processor::OnboardingProcessor;
use crate::managers::exceptions::application_base_exception::ApplicationResult;
use crate::managers::gateways::json_placeholder::json_placeholder_gateway::JsonPlaceholderGateway;
use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::models::user_profile::UserProfile;

// Onboarding is one task to the manager, with several kinds of work behind it: an external lookup, deciding what
// the customer looks like, saving it, and recording that it happened. The engine sequences those steps so the
// manager stays a list of one-line coordinations.
pub struct CustomerOnboardingEngine {
    json_placeholder_gateway: Arc<JsonPlaceholderGateway>,
    onboarding_processor: Arc<OnboardingProcessor>,
}

impl CustomerOnboardingEngine {
    pub fn new(
        json_placeholder_gateway: Arc<JsonPlaceholderGateway>,
        onboarding_processor: Arc<OnboardingProcessor>,
    ) -> Self {
        CustomerOnboardingEngine {
            json_placeholder_gateway,
            onboarding_processor,
        }
    }

    /// The engine is a shared singleton built by the ServiceLocator, which never sees the DataFacade, so the manager
    /// passes its DataFacade in with each call. (Rust modules may refer to each other, so the engine imports the
    /// DataFacade with no import cycle to avoid.)
    pub async fn onboard_customer(&self, user_id: i64, data_facade: &DataFacade) -> ApplicationResult<Customer> {
        let user_profile: UserProfile = self.json_placeholder_gateway.get_user_profile(user_id).await?;
        let customer: Customer = self.onboarding_processor.build_customer(&user_profile);

        data_facade.save_customer(&customer).await?;
        let onboarded_event: AuditEvent =
            self.onboarding_processor
                .build_onboarded_event(&customer, &user_profile, Utc::now());
        data_facade.save_audit_event(&onboarded_event).await?;
        Ok(customer)
    }
}
