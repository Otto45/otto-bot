from typing import final

from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.invalid_customer_exception import InvalidCustomerException
from example_app.managers.models.customer import Customer


@final
class CustomerProcessor:
    def check_customer(self, customer: Customer) -> None:
        """Raises InvalidCustomerException when the customer can't be saved as given. Its message is written for the
        person using the app, because a service answers a client error with it."""
        context: ContextualData = ContextualData().with_value("customer_id", customer.id)

        if customer.name.strip() == "":
            raise InvalidCustomerException("The customer's name is required", context)
        if "@" not in customer.email:
            raise InvalidCustomerException("The customer's email address is not valid", context)
