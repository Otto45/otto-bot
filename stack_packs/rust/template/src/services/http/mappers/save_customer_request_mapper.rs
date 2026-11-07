use crate::managers::models::customer::Customer;
use crate::services::http::wire_models::save_customer_request::SaveCustomerRequest;

pub struct SaveCustomerRequestMapper;

impl SaveCustomerRequestMapper {
    pub fn to_customer(customer_id: String, save_customer_request: SaveCustomerRequest) -> Customer {
        Customer {
            id: customer_id,
            name: save_customer_request.name,
            email: save_customer_request.email,
        }
    }
}
