#[must_use]
pub fn agent_string_is_insufficient_funds(message: &str) -> bool {
    let text = message
        .to_ascii_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'");
    [
        "openrouter billing/credit failure",
        "insufficient credits",
        "insufficient credit balance",
        "insufficient funds",
        "insufficient balance",
        "credit balance is too low",
        "no credits",
        "out of credits",
        "payment required",
        "you've hit your usage limit",
        "you're out of usage",
        "\"codexerrorinfo\":\"usagelimitexceeded\"",
        "upgrade your plan to continue",
    ]
    .iter()
    .any(|phrase| text.contains(phrase))
}

#[cfg(test)]
mod tests {
    use super::agent_string_is_insufficient_funds;

    #[test]
    fn explicit_payment_failures_only() {
        for message in [
            "Insufficient funds",
            "INSUFFICIENT CREDITS",
            "Insufficient credit balance",
            "Provider error: INSUFFICIENT CREDIT BALANCE. Please top up.",
            "Your credit balance is too low",
            "You have no credits remaining.",
            "no credits",
            "HTTP 402 Payment Required",
            "insufficient balance",
            "out of credits",
            "OpenRouter billing/credit failure",
            "You've hit your usage limit",
            "You\u{2019}re out of usage",
            "codex RPC error: {\"codexErrorInfo\":\"usageLimitExceeded\"}",
            "Upgrade your plan to continue",
        ] {
            assert!(agent_string_is_insufficient_funds(message), "{message}");
        }
        for message in [
            "rate limit exceeded",
            "authentication failed",
            "invalid model",
            "timeout",
        ] {
            assert!(!agent_string_is_insufficient_funds(message), "{message}");
        }
    }
}
