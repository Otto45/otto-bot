import asyncio
from typing import Final, final

import asyncpg
from pydantic import ValidationError

from example_app.managers.common.formatters.validation_error_formatter import ValidationErrorFormatter
from example_app.managers.data_layer.managers.gateways.relational_database.mappers.customer_mapper import (
    CustomerMapper,
)
from example_app.managers.data_layer.managers.gateways.relational_database.query_builders.customer_query_builder import (  # noqa: E501
    CustomerQueryBuilder,
)
from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.customer_row import (
    CustomerRow,
)
from example_app.managers.data_layer.managers.gateways.relational_database.resource_models.sql_query import SqlQuery
from example_app.managers.data_layer.managers.gateways.relational_database.validators.validator_customer_row import (
    ValidatorCustomerRow,
)
from example_app.managers.exceptions.contextual_data import ContextualData
from example_app.managers.exceptions.customer_not_found_exception import CustomerNotFoundException
from example_app.managers.exceptions.invalid_postgres_customer_row_exception import (
    InvalidPostgresCustomerRowException,
)
from example_app.managers.exceptions.postgres_query_exception import PostgresQueryException
from example_app.managers.models.customer import Customer

# How long a statement waits for a connection (opening one included): long enough for a database that sleeps to wake,
# short enough that one that can't be reached fails the request rather than hanging it.
_ACQUIRE_TIMEOUT_SECONDS: Final[float] = 10


@final
class RelationalDatabaseGateway:
    def __init__(self, pool: asyncpg.Pool) -> None:
        """Given a pool the seam built without connecting. The gateway starts it when it first sends a statement (with
        no connection open: it opens them as statements need them), and starts it once, however many statements are
        sent at the same time. The schema is not the gateway's: it is migrated outside the application (migrations/,
        applied by dbmate)."""
        self._pool: asyncpg.Pool = pool
        self._pool_lock: asyncio.Lock = asyncio.Lock()
        self._pool_started: bool = False

    async def get_customer(self, customer_id: str) -> Customer:
        context: ContextualData = ContextualData().with_value("customer_id", customer_id)
        sql_query: SqlQuery = CustomerQueryBuilder.build_get_customer_query(customer_id)

        records: list[asyncpg.Record] = await self._fetch(sql_query, "retrieving a customer", context)
        if not records:
            raise CustomerNotFoundException("The customer was not found", context)

        customer_row: CustomerRow = RelationalDatabaseGateway._to_customer_row(records[0], context)
        ValidatorCustomerRow.validate_customer_row(customer_row)
        return CustomerMapper.to_customer(customer_row)

    async def save_customer(self, customer: Customer) -> None:
        context: ContextualData = ContextualData().with_value("customer_id", customer.id)
        sql_query: SqlQuery = CustomerQueryBuilder.build_save_customer_query(customer)

        await self._execute(sql_query, "saving a customer", context)

    async def close(self) -> None:
        """Closes the pool's connections. A pool that was never started has none, and is left as it is."""
        async with self._pool_lock:
            if self._pool_started:
                await self._pool.close()

    # What asyncpg raises: Postgres's own errors, the driver's, and a connection that is refused or times out.
    async def _fetch(
        self, sql_query: SqlQuery, operation_description: str, context: ContextualData
    ) -> list[asyncpg.Record]:
        try:
            pool: asyncpg.Pool = await self._get_started_pool()
            async with pool.acquire(timeout=_ACQUIRE_TIMEOUT_SECONDS) as connection:
                return await connection.fetch(sql_query.statement, *sql_query.parameters)
        except (asyncpg.PostgresError, asyncpg.InterfaceError, OSError, TimeoutError) as error:
            raise RelationalDatabaseGateway._to_query_exception(
                error, sql_query, operation_description, context
            ) from error

    async def _execute(self, sql_query: SqlQuery, operation_description: str, context: ContextualData) -> None:
        try:
            pool: asyncpg.Pool = await self._get_started_pool()
            async with pool.acquire(timeout=_ACQUIRE_TIMEOUT_SECONDS) as connection:
                await connection.execute(sql_query.statement, *sql_query.parameters)
        except (asyncpg.PostgresError, asyncpg.InterfaceError, OSError, TimeoutError) as error:
            raise RelationalDatabaseGateway._to_query_exception(
                error, sql_query, operation_description, context
            ) from error

    async def _get_started_pool(self) -> asyncpg.Pool:
        async with self._pool_lock:
            if not self._pool_started:
                await self._pool
                self._pool_started = True
        return self._pool

    @staticmethod
    def _to_customer_row(record: asyncpg.Record, context: ContextualData) -> CustomerRow:
        try:
            return CustomerRow.model_validate(dict(record.items()))
        except ValidationError as error:
            raise InvalidPostgresCustomerRowException(
                "Customer row from the Postgres customers table could not be deserialized",
                context.with_value("errors", ValidationErrorFormatter.format(error, "row")),
            ) from error

    @staticmethod
    def _to_query_exception(
        driver_error: asyncpg.PostgresError | asyncpg.InterfaceError | OSError | TimeoutError,
        sql_query: SqlQuery,
        operation_description: str,
        context: ContextualData,
    ) -> PostgresQueryException:
        # Parameter values stay out of the context: they can hold customer data.
        exception_context: ContextualData = (
            context.with_value("error_type", type(driver_error).__name__)
            .with_value("postgres_error_message", str(driver_error))
            .with_value("sql_statement", sql_query.statement)
        )
        # Only set when the error comes from Postgres itself (e.g. not for a refused connection): its SQLSTATE code.
        if isinstance(driver_error, asyncpg.PostgresError):
            exception_context = exception_context.with_value("postgres_error_code", driver_error.sqlstate)

        return PostgresQueryException(f"Postgres query failed while {operation_description}", exception_context)
