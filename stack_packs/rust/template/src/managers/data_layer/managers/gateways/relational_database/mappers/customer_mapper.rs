use crate::managers::data_layer::managers::gateways::relational_database::resource_models::customer_row::CustomerRow;
use crate::managers::models::customer::Customer;

pub struct CustomerMapper;

impl CustomerMapper {
    /// Expects a row that has already passed ValidatorCustomerRow: each `expect` states a checked fact, and
    /// fails only if the validator is wrong.
    pub fn to_customer(customer_row: CustomerRow) -> Customer {
        Customer {
            id: customer_row.customer_id.expect("validated by ValidatorCustomerRow"),
            name: customer_row.full_name.expect("validated by ValidatorCustomerRow"),
            email: customer_row.email_address.expect("validated by ValidatorCustomerRow"),
        }
    }
}
