use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::exception_kind::ExceptionKind;
use crate::managers::models::customer::Customer;

pub struct CustomerProcessor;

impl CustomerProcessor {
    /// Fails with an InvalidCustomer exception when the customer can't be saved as given. Its message is written for
    /// the person using the app, because a service answers a client error with it.
    pub fn check_customer(&self, customer: &Customer) -> ApplicationResult<()> {
        let context: ContextualData = ContextualData::new().with("customer_id", customer.id.as_str());

        if customer.name.trim().is_empty() {
            return Err(ApplicationBaseException::new(
                ExceptionKind::InvalidCustomer,
                "The customer's name is required",
                context,
            ));
        }
        if !customer.email.contains('@') {
            return Err(ApplicationBaseException::new(
                ExceptionKind::InvalidCustomer,
                "The customer's email address is not valid",
                context,
            ));
        }
        Ok(())
    }
}
