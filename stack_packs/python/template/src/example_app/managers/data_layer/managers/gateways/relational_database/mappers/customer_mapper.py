from typing import Final, final

from example_app.managers.common.validators.validated_value import ValidatedValue
from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.customer_row import (
    CustomerRow,
)
from example_app.managers.models.customer import Customer

_VALIDATOR_NAME: Final[str] = "ValidatorCustomerRow"


@final
class CustomerMapper:
    @staticmethod
    def to_customer(customer_row: CustomerRow) -> Customer:
        """Expects a row that has already passed ValidatorCustomerRow: each ValidatedValue.expect states a checked fact,
        and fails only if the validator is wrong."""
        return Customer(
            id=ValidatedValue.expect(customer_row.customer_id, _VALIDATOR_NAME),
            name=ValidatedValue.expect(customer_row.full_name, _VALIDATOR_NAME),
            email=ValidatedValue.expect(customer_row.email_address, _VALIDATOR_NAME),
        )
