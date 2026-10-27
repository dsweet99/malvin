#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AccountDenial {
    UsageLimit,
    PlanUpgrade,
    Funds,
}

fn normalized_agent_text(message: &str) -> String {
    message
        .to_ascii_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'")
}

fn text_has_usage_words(text: &str) -> bool {
    text.contains("usage limit") || text.contains("you're out of usage")
}

fn text_has_usage_code(text: &str) -> bool {
    text.contains("\"codexerrorinfo\":\"usagelimitexceeded\"")
}

fn text_is_usage_limit(text: &str) -> bool {
    text_has_usage_words(text) || text_has_usage_code(text)
}

fn text_is_plan_upgrade(text: &str) -> bool {
    text.contains("upgrade your plan")
}

fn text_is_throughput_limit(text: &str) -> bool {
    text.contains("rate limit") || text.contains("requests per")
}

fn text_is_storage_quota(text: &str) -> bool {
    [
        "disk quota",
        "storage quota",
        "filesystem quota",
        "file system quota",
    ]
    .iter()
    .any(|marker| text.contains(marker))
}

fn text_has_payment_words(text: &str) -> bool {
    text.contains("credit")
        || text.contains("funds")
        || text.contains("billing")
        || text.contains("payment required")
}

fn text_has_word(text: &str, word: &str) -> bool {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|token| token == word)
}

fn text_has_shortfall(text: &str) -> bool {
    text.contains("insufficient")
        || text.contains("exceeded")
        || text.contains("too low")
        || text.contains("out of")
        || text.contains("no credit")
        || text.contains("more credit")
        || text_has_word(text, "exhausted")
        || text_has_word(text, "reached")
}

fn text_mentions_billable(text: &str) -> bool {
    text_has_payment_words(text) || text.contains("quota") || text.contains("balance")
}

fn text_has_named_funds_marker(text: &str) -> bool {
    text.contains("payment required") || text.contains("openrouter billing/credit failure")
}

fn text_is_funds_exhausted(text: &str) -> bool {
    if text_is_storage_quota(text) {
        return false;
    }
    if text_is_throughput_limit(text) && !text_has_payment_words(text) {
        return false;
    }
    if text_has_named_funds_marker(text) {
        return true;
    }
    text_mentions_billable(text) && text_has_shortfall(text)
}

#[must_use]
pub(crate) fn account_denial(message: &str) -> Option<AccountDenial> {
    let text = normalized_agent_text(message);
    if text_is_usage_limit(&text) {
        return Some(AccountDenial::UsageLimit);
    }
    if text_is_plan_upgrade(&text) {
        return Some(AccountDenial::PlanUpgrade);
    }
    if text_is_funds_exhausted(&text) {
        return Some(AccountDenial::Funds);
    }
    None
}

#[must_use]
pub(crate) fn agent_string_is_usage_limit(message: &str) -> bool {
    text_is_usage_limit(&normalized_agent_text(message))
}

#[must_use]
pub(crate) fn agent_string_is_upgrade_plan(message: &str) -> bool {
    text_is_plan_upgrade(&normalized_agent_text(message))
}

#[must_use]
pub fn agent_string_is_insufficient_funds(message: &str) -> bool {
    account_denial(message).is_some()
}

#[cfg(test)]
mod tests {
    use super::{AccountDenial, account_denial, agent_string_is_insufficient_funds};

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
            "Insufficient credit.",
            "Upgrade your plan to use this model",
            "This request requires more credits, or fewer max_tokens.",
            "You exceeded your current quota, please check your plan and billing details.",
            "Billing hard limit has been reached",
            "Quota exhausted",
        ] {
            assert!(agent_string_is_insufficient_funds(message), "{message}");
        }
        for message in [
            "rate limit exceeded",
            "authentication failed",
            "invalid model",
            "timeout",
            "usage is fine",
            "Upgrade your",
            "Rate limit reached for requests per min. Limit: 20 / min.",
            "You exceeded your current quota of requests per min.",
            "output cap reached while thinking; raise `context_size` for this model",
            "no credentials",
            "The balance was breached",
        ] {
            assert!(!agent_string_is_insufficient_funds(message), "{message}");
        }
    }

    #[test]
    fn storage_quota_failures_are_not_account_denials() {
        for message in [
            "Disk quota exceeded (os error 122)",
            "Provider error: DISK QUOTA EXCEEDED (os error 122)",
            "Storage quota exhausted",
            "Filesystem quota reached",
            "File system quota exceeded",
            "Failed to write billing.json: Disk quota exceeded (os error 122)",
        ] {
            assert_eq!(account_denial(message), None, "{message}");
        }
    }

    #[test]
    fn account_denial_names_the_payment_kind() {
        assert_eq!(
            account_denial("Usage limit exceeded"),
            Some(AccountDenial::UsageLimit)
        );
        assert_eq!(
            account_denial("Upgrade your plan to use this model"),
            Some(AccountDenial::PlanUpgrade)
        );
        assert_eq!(
            account_denial("Insufficient credit."),
            Some(AccountDenial::Funds)
        );
        assert_eq!(
            account_denial(
                "You exceeded your current quota, please check your plan and billing details."
            ),
            Some(AccountDenial::Funds)
        );
        assert_eq!(
            account_denial("Billing hard limit has been reached"),
            Some(AccountDenial::Funds)
        );
        assert_eq!(
            account_denial("Quota exhausted"),
            Some(AccountDenial::Funds)
        );
        assert_eq!(account_denial("rate limit exceeded"), None);
        assert_eq!(account_denial("The balance was breached"), None);
    }
}
