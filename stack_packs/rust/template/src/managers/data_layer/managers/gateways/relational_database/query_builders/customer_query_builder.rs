use crate::managers::data_layer::managers::gateways::relational_database::resource_models::sql_query::{
    SqlParameter, SqlQuery,
};
use crate::managers::models::customer::Customer;

pub struct CustomerQueryBuilder;

impl CustomerQueryBuilder {
    pub fn build_get_customer_query(customer_id: &str) -> SqlQuery {
        SqlQuery {
            statement: "SELECT customer_id, full_name, email_address FROM customers WHERE customer_id = $1",
            parameters: vec![SqlParameter::Text(customer_id.to_string())],
        }
    }

    pub fn build_save_customer_query(customer: &Customer) -> SqlQuery {
        SqlQuery {
            statement: concat!(
                "INSERT INTO customers (customer_id, full_name, email_address) ",
                "VALUES ($1, $2, $3) ",
                "ON CONFLICT (customer_id) DO UPDATE SET ",
                "full_name = excluded.full_name, email_address = excluded.email_address",
            ),
            parameters: vec![
                SqlParameter::Text(customer.id.clone()),
                SqlParameter::Text(customer.name.clone()),
                SqlParameter::Text(customer.email.clone()),
            ],
        }
    }
}
