from typing import final

from starlette.types import Message, Send


# Sends a request's response messages on, and remembers whether the response has started: once it has, an exception
# can no longer be answered with a response of its own.
@final
class ResponseStartTracker:
    def __init__(self, send: Send) -> None:
        self._send: Send = send
        self.started: bool = False

    async def send(self, message: Message) -> None:
        if message["type"] == "http.response.start":
            self.started = True
        await self._send(message)
