#[must_use]
pub(crate) fn router_wants_continue(agent_text: &str) -> bool {
    let trimmed = agent_text.trim();
    if trimmed == "CONTINUE_ROUTER" {
        return true;
    }
    trimmed
        .lines()
        .any(|line| line.trim() == "CONTINUE_ROUTER")
}

/// True for short greeting/social utterances that should not enter the coding router.
///
/// Uses a whole-utterance allowlist (not substring matching) so asks like
/// `"fix bug"` or `"Hello world program"` stay on the normal coding path.
#[must_use]
pub(crate) fn is_trivial_social_request(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.len() > 40 {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if trivial_social_has_code_cue(&lower) {
        return false;
    }
    let normalized = normalize_trivial_social_utterance(&lower);
    TRIVIAL_SOCIAL_ALLOWLIST.contains(&normalized.as_str())
}

const TRIVIAL_SOCIAL_ALLOWLIST: &[&str] = &[
    "hello",
    "hi",
    "hey",
    "thanks",
    "thank you",
    "thankyou",
    "good morning",
    "good afternoon",
    "good evening",
    "yo",
    "sup",
    "howdy",
];

fn trivial_social_has_code_cue(lower: &str) -> bool {
    const CUES: &[&str] = &[
        "/",
        "`",
        "::",
        ".rs",
        ".py",
        ".md",
        "fix ",
        "fix\n",
        "bug",
        "implement",
        "refactor",
        "test",
        "pr #",
        "src/",
        "http://",
        "https://",
    ];
    // Also reject leading "fix" / trailing code words without requiring a trailing space.
    if lower == "fix" || lower.starts_with("fix ") {
        return true;
    }
    CUES.iter().any(|cue| lower.contains(cue))
}

fn normalize_trivial_social_utterance(lower: &str) -> String {
    let mapped: String = lower
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c.is_whitespace() {
                c
            } else {
                ' '
            }
        })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_scored_line(agent_text: &str, prefix: &str) -> Option<String> {
    agent_text.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix(prefix)
            .map(|rest| rest.trim().to_string())
    })
}

/// # Errors
///
/// Returns an error when `COMPLEXITY_SCORE` is missing, malformed, or outside 1–10.
pub(crate) fn parse_complexity_score(agent_text: &str) -> Result<u8, String> {
    let value = parse_scored_line(agent_text, "COMPLEXITY_SCORE:").ok_or_else(|| {
        "router_a_1: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })?;
    let score: u32 = value.parse().map_err(|_| {
        "router_a_1: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })?;
    if !(1..=10).contains(&score) {
        return Err(
            "router_a_1: missing or malformed COMPLEXITY_SCORE in agent response".to_string(),
        );
    }
    u8::try_from(score).map_err(|_| {
        "router_a_1: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })
}

/// # Errors
///
/// Returns an error when `CODING_TASK` is missing or not `YES`/`NO`.
pub(crate) fn parse_coding_task(agent_text: &str) -> Result<bool, String> {
    let value = parse_scored_line(agent_text, "CODING_TASK:").ok_or_else(|| {
        "router_a_2: missing or malformed CODING_TASK in agent response".to_string()
    })?;
    match value.as_str() {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err("router_a_2: missing or malformed CODING_TASK in agent response".to_string()),
    }
}

#[cfg(test)]
mod router_wants_continue_tests {
    use super::router_wants_continue;

    #[test]
    fn exact_continue_marker() {
        assert!(router_wants_continue("CONTINUE_ROUTER"));
    }

    #[test]
    fn continue_marker_with_trailing_newlines() {
        assert!(router_wants_continue("CONTINUE_ROUTER\n\n"));
    }

    #[test]
    fn continue_marker_on_own_line() {
        assert!(router_wants_continue("CONTINUE_ROUTER\n"));
    }

    #[test]
    fn report_text_does_not_continue() {
        assert!(!router_wants_continue(
            "Summary\n\nEvidence shows the fix works.\n"
        ));
    }

    #[test]
    fn inline_continue_token_without_own_line_does_not_continue() {
        assert!(!router_wants_continue(
            "Please output CONTINUE_ROUTER when done."
        ));
    }
}

#[cfg(test)]
mod trivial_social_request_tests {
    use super::is_trivial_social_request;

    #[test]
    fn allowlist_greetings_match() {
        assert!(is_trivial_social_request("Hello"));
        assert!(is_trivial_social_request("hello!"));
        assert!(is_trivial_social_request("  hi  "));
        assert!(is_trivial_social_request("thanks"));
        assert!(is_trivial_social_request("Thank you"));
        assert!(is_trivial_social_request("good morning"));
    }

    #[test]
    fn coding_asks_are_not_trivial() {
        assert!(!is_trivial_social_request("fix bug"));
        assert!(!is_trivial_social_request("Fix the bug"));
        assert!(!is_trivial_social_request("hi, fix src/foo.rs"));
        assert!(!is_trivial_social_request("Hello world program"));
        assert!(!is_trivial_social_request("thanks for reviewing PR #12"));
        assert!(!is_trivial_social_request("implement the feature"));
    }

    #[test]
    fn empty_and_long_requests_are_not_trivial() {
        assert!(!is_trivial_social_request(""));
        assert!(!is_trivial_social_request("   "));
        assert!(!is_trivial_social_request(&"hello ".repeat(20)));
    }
}

#[cfg(test)]
mod parse_router_a_tokens_tests {
    use super::{parse_coding_task, parse_complexity_score};

    #[test]
    fn parse_complexity_score_from_own_line() {
        assert_eq!(
            parse_complexity_score("analysis\nCOMPLEXITY_SCORE: 3\n").expect("score"),
            3
        );
    }

    #[test]
    fn parse_complexity_score_inside_fenced_block() {
        let text = "done\n```\nCOMPLEXITY_SCORE: 7\n```\n";
        assert_eq!(parse_complexity_score(text).expect("score"), 7);
    }

    #[test]
    fn parse_complexity_score_rejects_out_of_range() {
        assert!(parse_complexity_score("COMPLEXITY_SCORE: 11\n").is_err());
        assert!(parse_complexity_score("COMPLEXITY_SCORE: 0\n").is_err());
    }

    #[test]
    fn parse_complexity_score_rejects_missing_token() {
        assert!(parse_complexity_score("no score here\n").is_err());
    }

    #[test]
    fn parse_coding_task_yes_and_no() {
        assert!(parse_coding_task("CODING_TASK: YES\n").expect("yes"));
        assert!(!parse_coding_task("CODING_TASK: NO\n").expect("no"));
    }

    #[test]
    fn parse_coding_task_rejects_malformed() {
        assert!(parse_coding_task("CODING_TASK: maybe\n").is_err());
        assert!(parse_coding_task("no token\n").is_err());
    }
}
