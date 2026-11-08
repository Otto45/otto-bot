from typing import final

from example_app.managers.models.greeting import Greeting


@final
class GreetingProcessor:
    def __init__(self, greeting_prefix: str) -> None:
        self._greeting_prefix: str = greeting_prefix

    def greet(self, name: str) -> Greeting:
        return Greeting(message=f"{self._greeting_prefix}, {name}!")
