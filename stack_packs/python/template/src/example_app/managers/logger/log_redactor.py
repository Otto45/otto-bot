import re
from typing import Final, final

_REDACTED: Final[str] = "[redacted]"

# What a log line must never carry, however it got there (an upstream error that quotes a URL, a library's message):
# each pattern and what it is replaced with, applied in order.
_PATTERNS: Final[tuple[tuple[re.Pattern[str], str], ...]] = (
    # The user and password in a URL (a connection string, a signed address)
    (re.compile(r"(?P<scheme>\b[a-zA-Z][a-zA-Z0-9+.-]*://)[^\s/@:]+:[^\s/@]+@"), rf"\g<scheme>{_REDACTED}@"),
    # An Authorization header's credentials
    (re.compile(r"\b(?P<scheme>bearer|basic)\s+[A-Za-z0-9._~+/=-]{8,}", re.IGNORECASE), rf"\g<scheme> {_REDACTED}"),
    # JSON Web Tokens (session cookies, access tokens)
    (re.compile(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+"), _REDACTED),
    # API keys: OpenAI's and Anthropic's (sk-), WorkOS's (sk_test_, sk_live_), Neon's (napi_), AWS access key ids
    (re.compile(r"\b(?:sk[-_][A-Za-z0-9_-]{16,}|napi_[A-Za-z0-9]{16,}|(?:AKIA|ASIA)[A-Z0-9]{16})\b"), _REDACTED),
    # A secret given by name: password=..., "token": "...", api_key: ...
    (
        re.compile(
            r"\b(?P<name>password|passwd|secret|token|api[_-]?key|access[_-]?key|client[_-]?secret)"
            r"""(?P<separator>["']?\s*[=:]\s*["']?)[^\s"'&,;}]+""",
            re.IGNORECASE,
        ),
        rf"\g<name>\g<separator>{_REDACTED}",
    ),
    # A secret in a query string: ?code=..., &signature=...
    (re.compile(r"""(?P<name>[?&](?:code|sig|signature|state)=)[^&\s#"']+""", re.IGNORECASE), rf"\g<name>{_REDACTED}"),
    # Email addresses: who someone is stays out of the logs (their ids are enough)
    (re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b"), _REDACTED),
)


# Takes out of a log line's text what must never be logged. The context never holds anything sensitive in the first
# place; this catches what reaches the logs anyway, in an exception's message or stack that quotes it.
@final
class LogRedactor:
    @staticmethod
    def redact(text: str) -> str:
        redacted: str = text
        for pattern, replacement in _PATTERNS:
            redacted = pattern.sub(replacement, redacted)
        return redacted
