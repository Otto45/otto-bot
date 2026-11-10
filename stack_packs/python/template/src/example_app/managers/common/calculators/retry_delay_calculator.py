import random
from typing import Final, final

# How long a gateway waits before sending a failed request again: doubling from the first, up to the longest, with up
# to a quarter of a second added at random so requests that failed together aren't sent together. The longest a
# service's own Retry-After is waited for; asking for longer is treated as this.
_FIRST_RETRY_DELAY_SECONDS: Final[float] = 1
_LONGEST_RETRY_DELAY_SECONDS: Final[float] = 20
_MAX_RETRY_JITTER_SECONDS: Final[float] = 0.25
_LONGEST_RETRY_AFTER_SECONDS: Final[float] = 60


@final
class RetryDelayCalculator:
    """The waits between a gateway's retries, the same for every gateway: a gateway that retries keeps its own loop (it
    decides on its own exceptions) and asks this how long to wait."""

    @staticmethod
    def get_retry_delay_seconds(retry: int, retry_after_seconds: int | None) -> float:
        """The retry-th wait (the first retry's is 1): what the service asked for (retry_after_seconds, from its
        Retry-After, up to a minute), or doubling from a second up to 20, whichever is longer, with jitter."""
        backoff: float = min(_FIRST_RETRY_DELAY_SECONDS * 2 ** (retry - 1), _LONGEST_RETRY_DELAY_SECONDS)
        asked: float = (
            min(float(retry_after_seconds), _LONGEST_RETRY_AFTER_SECONDS) if retry_after_seconds is not None else 0
        )
        return max(backoff, asked) + random.uniform(0, _MAX_RETRY_JITTER_SECONDS)

    @staticmethod
    def parse_retry_after(retry_after: str) -> int | None:
        """A Retry-After header's wait, in whole seconds; None for one that isn't a number of seconds (an HTTP date,
        which is then waited out by the backoff instead)."""
        stripped: str = retry_after.strip()
        return int(stripped) if stripped.isdigit() else None
