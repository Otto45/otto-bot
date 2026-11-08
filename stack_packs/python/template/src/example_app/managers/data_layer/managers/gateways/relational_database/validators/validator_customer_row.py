from typing import final

from example_app.managers.common.validators.validator_string import ValidatorString
from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.customer_row import (
    CustomerRow,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_postgres_customer_row_exception import (
    InvalidPostgresCustomerRowException,
)


@final
class ValidatorCustomerRow:
    @staticmethod
    def validate_customer_row(customer_row: CustomerRow) -> None:
        errors: list[str] = [
            error
            for error in (
                ValidatorString.validate("customer_id", customer_row.customer_id),
                ValidatorString.validate("full_name", customer_row.full_name),
                ValidatorString.validate("email_address", customer_row.email_address),
            )
            if error is not None
        ]

        if errors:
            raise InvalidPostgresCustomerRowException(
                "Customer row from the Postgres customers table failed validation",
                ContextualData().with_value("resource_name", "customer").with_value("errors", "; ".join(errors)),
            )
