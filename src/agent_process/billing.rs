fn normalized_agent_text(message: &str) -> String {
    message
        .to_ascii_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'")
}

fn text_is_usage_limit(text: &str) -> bool {
    text.contains("usage limit")
        || text.contains("you're out of usage")
        || text.contains("\"codexerrorinfo\":\"usagelimitexceeded\"")
}

#[must_use]
pub(crate) fn agent_string_is_usage_limit(message: &str) -> bool {
    text_is_usage_limit(&normalized_agent_text(message))
}

#[must_use]
pub fn agent_string_is_insufficient_funds(message: &str) -> bool {
    let text = normalized_agent_text(message);
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
        "upgrade your plan to continue",
    ]
    .iter()
    .any(|phrase| text.contains(phrase))
        || text_is_usage_limit(&text)
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
            "Usage limit exceeded",
            "Your usage limit has been reached",
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
            "usage is fine",
        ] {
            assert!(!agent_string_is_insufficient_funds(message), "{message}");
        }
    }
}
