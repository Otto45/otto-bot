use std::sync::LazyLock;

use regex::Regex;

const REDACTED: &str = "[redacted]";

// What a log line must never carry, however it got there (an upstream error that quotes a URL, a library's message):
// each pattern and what it is replaced with, applied in order.
static PATTERNS: LazyLock<Vec<(Regex, String)>> = LazyLock::new(|| {
    let patterns: [(&str, String); 7] = [
        // The user and password in a URL (a connection string, a signed address)
        (
            r"(?P<scheme>\b[a-zA-Z][a-zA-Z0-9+.-]*://)[^\s/@:]+:[^\s/@]+@",
            format!("${{scheme}}{REDACTED}@"),
        ),
        // An Authorization header's credentials
        (
            r"(?i)\b(?P<scheme>bearer|basic)\s+[A-Za-z0-9._~+/=-]{8,}",
            format!("${{scheme}} {REDACTED}"),
        ),
        // JSON Web Tokens (session cookies, WorkOS's access tokens)
        (
            r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+",
            REDACTED.to_string(),
        ),
        // API keys: OpenAI's and Anthropic's (sk-), WorkOS's (sk_test_, sk_live_), Neon's (napi_), AWS access key ids
        (
            r"\b(?:sk[-_][A-Za-z0-9_-]{16,}|napi_[A-Za-z0-9]{16,}|(?:AKIA|ASIA)[A-Z0-9]{16})\b",
            REDACTED.to_string(),
        ),
        // A secret given by name: password=..., "token": "...", api_key: ...
        (
            concat!(
                r"(?i)\b(?P<name>password|passwd|secret|token|api[_-]?key|access[_-]?key|client[_-]?secret)",
                r#"(?P<separator>["']?\s*[=:]\s*["']?)[^\s"'&,;}]+"#,
            ),
            format!("${{name}}${{separator}}{REDACTED}"),
        ),
        // A secret in a query string: ?code=..., &signature=...
        (
            r#"(?i)(?P<name>[?&](?:code|sig|signature|state)=)[^&\s#"']+"#,
            format!("${{name}}{REDACTED}"),
        ),
        // Email addresses: who someone is stays out of the logs (their ids are enough)
        (
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b",
            REDACTED.to_string(),
        ),
    ];

    patterns
        .into_iter()
        .map(|(pattern, replacement): (&str, String)| {
            (
                Regex::new(pattern).expect("every redaction pattern compiles"),
                replacement,
            )
        })
        .collect()
});

// Takes out of a log line's text what must never be logged. The context never holds anything sensitive in the first
// place; this catches what reaches the logs anyway, in an error's message or stack that quotes it.
pub struct LogRedactor;

impl LogRedactor {
    pub fn redact(text: &str) -> String {
        let mut redacted: String = text.to_string();
        for (pattern, replacement) in PATTERNS.iter() {
            redacted = pattern.replace_all(&redacted, replacement.as_str()).into_owned();
        }
        redacted
    }
}
