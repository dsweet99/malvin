#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AccountDenial {
    UsageLimit,
    PlanUpgrade,
    Funds,
}

impl AccountDenial {
    #[must_use]
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::UsageLimit => "a usage limit",
            Self::PlanUpgrade => "a required plan upgrade",
            Self::Funds => "insufficient funds",
        }
    }
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

const PAYMENT_WORDS: &[&str] = &["credit", "credits", "funds", "billing"];
const QUOTA_WORDS: &[&str] = &["quota", "balance"];
const SHORTFALL_WORDS: &[&str] = &["insufficient", "exceeded", "exhausted", "reached", "low"];
const SHORTFALL_PREFIXES: &[&str] = &["no", "more", "out"];
const FILLER_WORDS: &[&str] = &[
    "your",
    "the",
    "current",
    "monthly",
    "account",
    "api",
    "available",
    "remaining",
    "spending",
    "prepaid",
    "hard",
    "limit",
    "has",
    "have",
    "been",
    "is",
    "are",
    "was",
    "of",
    "or",
    "too",
    "its",
];
const FUNDS_MARKERS: &[&str] = &[
    "payment required",
    "openrouter billing/credit failure",
    "resource has been exhausted",
];
const FUNDS_ERROR_CODES: &[&str] = &[
    "insufficient_quota",
    "insufficient_funds",
    "insufficient_balance",
    "insufficient_credits",
];

fn clauses(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| ",;:!?\n\"(){}[]".contains(c))
        .flat_map(|part| part.split(". "))
}

fn clause_words(clause: &str) -> impl Iterator<Item = &str> {
    clause
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_ascii_alphanumeric()))
}

fn text_has_payment_words(text: &str) -> bool {
    text.contains("payment required")
        || clauses(text)
            .flat_map(clause_words)
            .any(|word| PAYMENT_WORDS.contains(&word))
}

fn is_billable_word(word: &str) -> bool {
    PAYMENT_WORDS.contains(&word) || QUOTA_WORDS.contains(&word)
}

fn word_pair_reports_shortfall(first: &str, second: &str) -> bool {
    let shortfall_first = SHORTFALL_WORDS.contains(&first) || SHORTFALL_PREFIXES.contains(&first);
    (shortfall_first && is_billable_word(second))
        || (is_billable_word(first) && SHORTFALL_WORDS.contains(&second))
}

fn clause_reports_shortfall(clause: &str) -> bool {
    let words: Vec<&str> = clause_words(clause)
        .filter(|word| !FILLER_WORDS.contains(word))
        .collect();
    words.iter().any(|word| FUNDS_ERROR_CODES.contains(word))
        || words
            .windows(2)
            .any(|pair| word_pair_reports_shortfall(pair[0], pair[1]))
}

fn text_has_named_funds_marker(text: &str) -> bool {
    FUNDS_MARKERS.iter().any(|marker| text.contains(marker))
}

fn text_is_funds_exhausted(text: &str) -> bool {
    if text_is_storage_quota(text) {
        return false;
    }
    if text_is_throughput_limit(text) && !text_has_payment_words(text) {
        return false;
    }
    text_has_named_funds_marker(text) || clauses(text).any(clause_reports_shortfall)
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

    #[test]
    fn transient_failures_near_billing_words_are_not_account_denials() {
        for message in [
            "HTTP 503 from load balancer: upstream timeout exceeded",
            "connection to load balancer timed out: retries exceeded",
            "stream error: max output tokens exceeded in /srv/billing-service/src/main.rs",
            "Claude bridge exited while reading /home/me/credit-risk/app.py: maximum buffer size exceeded",
            "Provider error: context length exceeded for model credit-scoring-v2",
            "Provider error: context length exceeded for model credit scoring",
            "gate failed: test_rate_limit_quota out of memory",
            "gate failed: test_insufficient_funds",
            "codex: socket closed, out of file descriptors; quota service unavailable",
            "max retries exceeded while contacting billing service",
            "credit check timed out",
            "Request timeout exceeded. Billing unavailable",
            "rate limit exceeded for /srv/credit-api",
        ] {
            assert_eq!(account_denial(message), None, "{message}");
        }
    }

    #[test]
    fn provider_funds_messages_stay_account_denials() {
        for message in [
            "Your credit balance is too low to access the Anthropic API.",
            "This request requires more credits, or fewer max_tokens. You requested up to 4096 tokens",
            "Insufficient Balance",
            "{\"error\":{\"code\":\"insufficient_quota\",\"message\":\"see docs\"}}",
            "{\"error\":{\"message\":\"Insufficient credits\",\"code\":402}}",
            "Your team has used all available credits or reached its monthly spending limit",
            "Credit limit exceeded",
            "Your credits have been exhausted.",
            "Your balance is insufficient.",
            "Rate limit exceeded: insufficient credits",
            "Your account has run out of prepaid credits",
            "Resource has been exhausted (e.g. check quota).",
        ] {
            assert_eq!(
                account_denial(message),
                Some(AccountDenial::Funds),
                "{message}"
            );
        }
    }

    #[test]
    fn denial_reasons_name_the_actual_denial() {
        assert_eq!(AccountDenial::UsageLimit.reason(), "a usage limit");
        assert_eq!(
            AccountDenial::PlanUpgrade.reason(),
            "a required plan upgrade"
        );
        assert_eq!(AccountDenial::Funds.reason(), "insufficient funds");
    }
}
