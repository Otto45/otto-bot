from typing import final

from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.sql_query import SqlQuery
from example_app.managers.models.customer import Customer


@final
class CustomerQueryBuilder:
    @staticmethod
    def build_get_customer_query(customer_id: str) -> SqlQuery:
        return SqlQuery(
            statement="SELECT customer_id, full_name, email_address FROM customers WHERE customer_id = $1",
            parameters=(customer_id,),
        )

    @staticmethod
    def build_save_customer_query(customer: Customer) -> SqlQuery:
        return SqlQuery(
            statement=(
                "INSERT INTO customers (customer_id, full_name, email_address) "
                "VALUES ($1, $2, $3) "
                "ON CONFLICT (customer_id) DO UPDATE SET "
                "full_name = excluded.full_name, email_address = excluded.email_address"
            ),
            parameters=(customer.id, customer.name, customer.email),
        )
