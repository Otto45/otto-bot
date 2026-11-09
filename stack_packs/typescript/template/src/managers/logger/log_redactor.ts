const REDACTED: string = "[redacted]";

// What a log line must never carry, however it got there (an upstream error that quotes a URL, a library's message):
// each pattern and what it is replaced with, applied in order.
const PATTERNS: readonly (readonly [RegExp, string])[] = [
  // The user and password in a URL (a connection string, a signed address)
  [/(?<scheme>\b[a-zA-Z][a-zA-Z0-9+.-]*:\/\/)[^\s/@:]+:[^\s/@]+@/g, `$<scheme>${REDACTED}@`],
  // An Authorization header's credentials
  [/\b(?<scheme>bearer|basic)\s+[A-Za-z0-9._~+/=-]{8,}/gi, `$<scheme> ${REDACTED}`],
  // JSON Web Tokens (session cookies, access tokens)
  [/\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/g, REDACTED],
  // API keys: OpenAI's and Anthropic's (sk-), WorkOS's (sk_test_, sk_live_), Neon's (napi_), AWS access key ids
  [/\b(?:sk[-_][A-Za-z0-9_-]{16,}|napi_[A-Za-z0-9]{16,}|(?:AKIA|ASIA)[A-Z0-9]{16})\b/g, REDACTED],
  // A secret given by name: password=..., "token": "...", api_key: ...
  [
    /\b(?<name>password|passwd|secret|token|api[_-]?key|access[_-]?key|client[_-]?secret)(?<separator>["']?\s*[=:]\s*["']?)[^\s"'&,;}]+/gi,
    `$<name>$<separator>${REDACTED}`,
  ],
  // A secret in a query string: ?code=..., &signature=...
  [/(?<name>[?&](?:code|sig|signature|state)=)[^&\s#"']+/gi, `$<name>${REDACTED}`],
  // Email addresses: who someone is stays out of the logs (their ids are enough)
  [/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b/g, REDACTED],
];

// Takes out of a log line's text what must never be logged. The context never holds anything sensitive in the first
// place; this catches what reaches the logs anyway, in an exception's message or stack that quotes it.
export class LogRedactor {
  public static redact(text: string): string {
    let redacted: string = text;
    for (const [pattern, replacement] of PATTERNS) {
      redacted = redacted.replace(pattern, replacement);
    }
    return redacted;
  }
}
