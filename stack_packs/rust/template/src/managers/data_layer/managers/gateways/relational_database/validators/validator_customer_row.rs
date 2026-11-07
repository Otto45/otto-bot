use crate::managers::common::validators::validator_string::ValidatorString;
use crate::managers::data_layer::managers::gateways::relational_database::resource_models::customer_row::CustomerRow;
use crate::managers::exceptions::application_base_exception::{ApplicationBaseException, ApplicationResult};
use crate::managers::exceptions::contextual_data::ContextualData;
use crate::managers::exceptions::relational_database_gateway_exception::RelationalDatabaseGatewayException;

pub struct ValidatorCustomerRow;

impl ValidatorCustomerRow {
    pub fn validate_customer_row(customer_row: &CustomerRow) -> ApplicationResult<()> {
        let errors: Vec<String> = [
            ("customer_id", customer_row.customer_id.as_deref()),
            ("full_name", customer_row.full_name.as_deref()),
            ("email_address", customer_row.email_address.as_deref()),
        ]
        .into_iter()
        .filter_map(|(column_name, column_value): (&str, Option<&str>)| {
            ValidatorString::validate(column_name, column_value)
        })
        .collect();

        if !errors.is_empty() {
            return Err(ApplicationBaseException::new(
                RelationalDatabaseGatewayException::InvalidPostgresCustomerRow,
                "Customer row from the Postgres customers table failed validation",
                ContextualData::new()
                    .with("resource_name", "customer")
                    .with("errors", errors.join("; ")),
            ));
        }
        Ok(())
    }
}
