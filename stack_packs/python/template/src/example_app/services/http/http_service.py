from typing import Any, final

import uvicorn
from fastapi import FastAPI
from fastapi.exceptions import RequestValidationError

from example_app.example_app_facade import ExampleAppFacade
from example_app.managers.exceptions.application_base_exception import ApplicationBaseException
from example_app.managers.logger.application_logger import ApplicationLogger
from example_app.managers.logger.uncaught_exception_logger import UncaughtExceptionLogger
from example_app.services.http.exception_handlers.application_exception_handler import ApplicationExceptionHandler
from example_app.services.http.exception_handlers.request_validation_exception_handler import (
    RequestValidationExceptionHandler,
)
from example_app.services.http.middleware.unexpected_error_middleware import UnexpectedErrorMiddleware
from example_app.services.http.routers.customer_router import CustomerRouter
from example_app.services.http.routers.user_profile_router import UserProfileRouter


# A service runs and listens. It sits outside the domain container and holds no application logic: it works the way its
# framework does, hands the Domain Facade only domain models, maps the domain models it gets back to response models,
# and translates the facade's exceptions into HTTP errors (ApplicationExceptionHandler).
@final
class HttpService:
    @staticmethod
    async def serve(host: str, port: int) -> None:
        """Creates the Domain Facade, serves requests with it until Ctrl-C or SIGTERM (finishing the requests under
        way), then closes it. The facade is created once and lives as long as the server: every request gets this one
        instance (ExampleAppFacadeDependency), and it is closed, with everything the locator cached, when the server has
        stopped, whether serving failed or not."""
        UncaughtExceptionLogger.install_for_event_loop()

        # e.g. invalid configuration, which fails while the facade is created, before the server listens
        async with ExampleAppFacade() as example_app_facade:
            # uvicorn logs through Python's logging, which main routes to the ApplicationLogger (log_config=None keeps
            # uvicorn from installing its own handlers).
            server: uvicorn.Server = uvicorn.Server(
                uvicorn.Config(
                    HttpService.get_application(example_app_facade),
                    host=host,
                    port=port,
                    log_config=None,
                    access_log=False,
                )
            )
            ApplicationLogger.info("The service is starting")
            await server.serve()

    @staticmethod
    def get_application(example_app_facade: ExampleAppFacade | None) -> FastAPI:
        """The application; without a facade, only for its contract (it can't answer requests)."""
        application: FastAPI = FastAPI(title="Example App", version="0.1.0")
        application.state.example_app_facade = example_app_facade
        application.include_router(CustomerRouter.create_router())
        application.include_router(UserProfileRouter.create_router())
        application.add_exception_handler(ApplicationBaseException, ApplicationExceptionHandler.handle)
        application.add_exception_handler(RequestValidationError, RequestValidationExceptionHandler.handle)
        application.add_middleware(UnexpectedErrorMiddleware)
        return application

    # The OpenAPI document is JSON of FastAPI's own making, typed Any.
    @staticmethod
    def get_contract() -> dict[str, Any]:
        """The service's contract, built from the routers. Building it creates no facade, so it needs no
        configuration."""
        return HttpService.get_application(None).openapi()
