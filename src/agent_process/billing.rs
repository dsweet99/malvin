#[must_use]
pub fn agent_string_is_insufficient_funds(message: &str) -> bool {
    let text = message.to_ascii_lowercase();
    [
        "openrouter billing/credit failure",
        "insufficient credits",
        "insufficient funds",
        "insufficient balance",
        "credit balance is too low",
        "no credits remaining",
        "out of credits",
        "payment required",
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
            "Your credit balance is too low",
            "You have no credits remaining.",
            "HTTP 402 Payment Required",
            "insufficient balance",
            "out of credits",
            "OpenRouter billing/credit failure",
        ] {
            assert!(agent_string_is_insufficient_funds(message), "{message}");
        }
        for message in [
            "rate limit exceeded",
            "you've hit your usage limit",
            "authentication failed",
            "invalid model",
            "timeout",
            "upgrade your plan to continue",
        ] {
            assert!(!agent_string_is_insufficient_funds(message), "{message}");
        }
    }
}
