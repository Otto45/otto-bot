from typing import final

from fastapi import Request

from example_app.example_app_facade import ExampleAppFacade


# The facade is created once, by HttpService.serve, and lives as long as the server. This only hands that one instance
# to a route (FastAPI's Depends); it never builds a facade, which would create and close every client and connection on
# each request.
@final
class ExampleAppFacadeDependency:
    @staticmethod
    def get(request: Request) -> ExampleAppFacade:
        example_app_facade: object = request.app.state.example_app_facade
        if not isinstance(example_app_facade, ExampleAppFacade):
            raise TypeError("The service was started without its Domain Facade")
        return example_app_facade
