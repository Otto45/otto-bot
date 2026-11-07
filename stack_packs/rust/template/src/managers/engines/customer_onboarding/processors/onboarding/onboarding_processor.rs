use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::managers::models::audit_event::AuditEvent;
use crate::managers::models::customer::Customer;
use crate::managers::models::user_profile::UserProfile;

// Belongs to the CustomerOnboardingEngine: only that engine uses it, so it lives in the engine's folder.
pub struct OnboardingProcessor;

impl OnboardingProcessor {
    pub fn build_customer(&self, user_profile: &UserProfile) -> Customer {
        Customer {
            id: format!("user-{}", user_profile.id),
            name: user_profile.name.clone(),
            email: user_profile.email.clone(),
        }
    }

    pub fn build_onboarded_event(
        &self,
        customer: &Customer,
        user_profile: &UserProfile,
        occurred_on: DateTime<Utc>,
    ) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4().simple().to_string(),
            customer_id: customer.id.clone(),
            description: format!("Customer was onboarded from user profile {}", user_profile.username),
            occurred_on,
        }
    }
}
