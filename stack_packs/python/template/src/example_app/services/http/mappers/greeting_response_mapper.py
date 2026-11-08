from typing import final

from example_app.managers.models.greeting import Greeting
from example_app.services.http.wire_models.greeting_response import GreetingResponse


@final
class GreetingResponseMapper:
    @staticmethod
    def to_greeting_response(greeting: Greeting) -> GreetingResponse:
        return GreetingResponse(message=greeting.message)
