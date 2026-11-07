use std::sync::LazyLock;

use regex::Regex;

const REDACTED: &str = "[REDACTED]";

// Secrets with a recognizable shape: API keys and tokens of well-known services, and private keys. Narrow on
// purpose: code is sent to the model as it is, so only what is certainly a secret is taken out.
static SECRET_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"sk-(?:proj-|svcacct-|admin-)?[A-Za-z0-9_-]{20,}",
        r"\bAKIA[0-9A-Z]{16}\b",
        r"\bgh[pousr]_[A-Za-z0-9]{36,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{22,}\b",
        r"\bxox[abpr]-[A-Za-z0-9-]{10,}\b",
        r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
    ]
    .into_iter()
    .map(|pattern: &str| Regex::new(pattern).expect("a secret pattern is a valid regular expression"))
    .collect()
});

// Takes secrets out of what is sent to the model (files it reads, what checks print), so a key that sits in the
// workspace never leaves the machine.
pub struct SecretRedactor;

impl SecretRedactor {
    pub fn redact(text: &str) -> String {
        let mut redacted: String = text.to_string();
        for secret_pattern in SECRET_PATTERNS.iter() {
            if secret_pattern.is_match(&redacted) {
                redacted = secret_pattern.replace_all(&redacted, REDACTED).into_owned();
            }
        }
        redacted
    }
}
