from typing import final

from example_app.managers.models.customer import Customer
from example_app.services.http.wire_models.save_customer_request import SaveCustomerRequest


@final
class SaveCustomerRequestMapper:
    @staticmethod
    def to_customer(customer_id: str, save_customer_request: SaveCustomerRequest) -> Customer:
        return Customer(id=customer_id, name=save_customer_request.name, email=save_customer_request.email)
