use std::sync::LazyLock;

use regex::{Captures, Regex};

const REDACTED: &str = "[REDACTED]";

// Secrets with a recognizable shape: API keys and tokens of well-known services, signed tokens, and private keys.
// Narrow on purpose: code is sent to the model as it is, so only what is certainly a secret is taken out.
static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"sk-(?:proj-|svcacct-|admin-)?[A-Za-z0-9_-]{20,}",
        r"\b[sr]k_live_[A-Za-z0-9]{16,}\b",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\bAIza[0-9A-Za-z_-]{35}\b",
        r"\bgh[pousr]_[A-Za-z0-9]{36,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{22,}\b",
        r"\bxox[abpr]-[A-Za-z0-9-]{10,}\b",
        r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}",
        r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
    ]
    .into_iter()
    .map(|pattern: &str| Regex::new(pattern).expect("a secret pattern is a valid regular expression"))
    .collect()
});

// The user and password in a URL (a connection string), and the host they are for.
static URL_CREDENTIALS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?P<scheme>\b[a-zA-Z][a-zA-Z0-9+.-]*://)[^\s/@:]+:[^\s/@]+@(?P<host>\[[^\]\s]*\]|[^\s/:?#]*)")
        .expect("the URL credentials pattern is a valid regular expression")
});

// Takes secrets out of what is sent to the model (files it reads, the diff a review is given, what checks print), so
// a key that sits in the workspace never leaves the machine.
pub struct SecretRedactor;

impl SecretRedactor {
    pub fn redact(text: &str) -> String {
        let mut redacted: String = text.to_string();
        for secret_pattern in SECRET_PATTERNS.iter() {
            if secret_pattern.is_match(&redacted) {
                redacted = secret_pattern.replace_all(&redacted, REDACTED).into_owned();
            }
        }
        if URL_CREDENTIALS_PATTERN.is_match(&redacted) {
            redacted = URL_CREDENTIALS_PATTERN
                .replace_all(&redacted, |captures: &Captures<'_>| -> String {
                    // A database on the developer's own machine is the default an app's code is written with: its
                    // password is no secret, and the model patches that code, so it is sent as it is.
                    if SecretRedactor::is_local_host(&captures["host"]) {
                        captures[0].to_string()
                    } else {
                        format!("{}{REDACTED}@{}", &captures["scheme"], &captures["host"])
                    }
                })
                .into_owned();
        }
        redacted
    }

    fn is_local_host(host: &str) -> bool {
        let host: String = host.to_ascii_lowercase();
        host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host.ends_with(".localhost")
    }
}
