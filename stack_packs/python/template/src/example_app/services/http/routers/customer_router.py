from http import HTTPStatus
from typing import final

from fastapi import APIRouter

from example_app.managers.models.audit_event import AuditEvent
from example_app.managers.models.customer import Customer
from example_app.managers.models.greeting import Greeting
from example_app.services.http.dependencies.example_app_facade_parameter import ExampleAppFacadeParameter
from example_app.services.http.mappers.audit_event_response_mapper import AuditEventResponseMapper
from example_app.services.http.mappers.greeting_response_mapper import GreetingResponseMapper
from example_app.services.http.mappers.save_customer_request_mapper import SaveCustomerRequestMapper
from example_app.services.http.wire_models.audit_event_list_response import AuditEventListResponse
from example_app.services.http.wire_models.error_response import ErrorResponse
from example_app.services.http.wire_models.greeting_response import GreetingResponse
from example_app.services.http.wire_models.save_customer_request import SaveCustomerRequest


# Each route maps the request into domain models, calls one facade operation, and maps the domain models it returns to
# a response model. Domain models never cross the wire in either direction. Each route also names the errors it can
# answer with (`responses`), so they are part of the service's contract (openapi.json). The facade comes from
# ExampleAppFacadeDependency, which hands out the one facade the service created; nothing builds one.
@final
class CustomerRouter:
    @staticmethod
    def create_router() -> APIRouter:
        router: APIRouter = APIRouter()
        router.add_api_route(
            "/customers/{customer_id}",
            CustomerRouter.save_customer,
            methods=["PUT"],
            status_code=HTTPStatus.NO_CONTENT,
            responses={
                HTTPStatus.NO_CONTENT: {"description": "The customer was saved"},
                HTTPStatus.BAD_REQUEST: {"description": "The customer can't be saved as given", "model": ErrorResponse},
                HTTPStatus.UNPROCESSABLE_CONTENT: {
                    "description": "The body does not fit the request model",
                    "model": ErrorResponse,
                },
            },
        )
        router.add_api_route(
            "/customers/{customer_id}/greetings",
            CustomerRouter.create_customer_greeting,
            methods=["POST"],
            status_code=HTTPStatus.CREATED,
            responses={
                HTTPStatus.CREATED: {"description": "The customer was greeted"},
                HTTPStatus.NOT_FOUND: {"description": "No customer has this id", "model": ErrorResponse},
                HTTPStatus.UNPROCESSABLE_CONTENT: {
                    "description": "The request does not fit the route",
                    "model": ErrorResponse,
                },
            },
        )
        router.add_api_route(
            "/customers/{customer_id}/audit-events",
            CustomerRouter.get_customer_audit_events,
            methods=["GET"],
            responses={
                HTTPStatus.OK: {"description": "The customer's audit events, oldest first"},
                HTTPStatus.UNPROCESSABLE_CONTENT: {
                    "description": "The request does not fit the route",
                    "model": ErrorResponse,
                },
            },
        )
        return router

    @staticmethod
    async def save_customer(
        customer_id: str, save_customer_request: SaveCustomerRequest, example_app_facade: ExampleAppFacadeParameter
    ) -> None:
        customer: Customer = SaveCustomerRequestMapper.to_customer(customer_id, save_customer_request)

        await example_app_facade.save_customer(customer)

    @staticmethod
    async def create_customer_greeting(
        customer_id: str, example_app_facade: ExampleAppFacadeParameter
    ) -> GreetingResponse:
        greeting: Greeting = await example_app_facade.get_customer_greeting(customer_id)

        return GreetingResponseMapper.to_greeting_response(greeting)

    @staticmethod
    async def get_customer_audit_events(
        customer_id: str, example_app_facade: ExampleAppFacadeParameter
    ) -> AuditEventListResponse:
        audit_events: list[AuditEvent] = await example_app_facade.get_customer_audit_events(customer_id)

        return AuditEventResponseMapper.to_audit_event_list_response(audit_events)
