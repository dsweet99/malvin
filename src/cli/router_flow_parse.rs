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

/// True for short greetings that should not enter the coding router (whole-utterance allowlist).
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
    "hello", "hi", "hey", "thanks", "thank you", "thankyou", "good morning", "good afternoon",
    "good evening", "yo", "sup", "howdy",
];

fn trivial_social_has_code_cue(lower: &str) -> bool {
    const CUES: &[&str] = &[
        "/", "`", "::", ".rs", ".py", ".md", "fix ", "fix\n", "bug", "implement", "refactor",
        "test", "pr #", "src/", "http://", "https://",
    ];
    lower == "fix" || lower.starts_with("fix ") || CUES.iter().any(|cue| lower.contains(cue))
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

const SCORE_LABEL_DECORATION: &[char] = &['*', '`', '_', '"', '\'', ' ', '\t'];

/// Mid-line `LABEL`/`LABEL:`; last valid match wins (tolerates markdown / spaces around `:`).
fn last_labeled_value<T>(
    agent_text: &str,
    label: &str,
    mut parse: impl FnMut(&str) -> Option<T>,
) -> Option<T> {
    let mut from = 0;
    let mut best = None;
    while let Some(rel) = agent_text[from..].find(label) {
        let start = from + rel;
        from = start + label.len();
        if labeled_token_has_alnum_prefix(agent_text, start) {
            continue;
        }
        let mut rest = &agent_text[start + label.len()..];
        rest = rest.trim_start_matches(SCORE_LABEL_DECORATION);
        let Some(after_colon) = rest.strip_prefix(':') else {
            continue;
        };
        let value = after_colon.trim_start_matches(SCORE_LABEL_DECORATION);
        if value.is_empty() {
            continue;
        }
        if let Some(parsed) = parse(value) {
            best = Some(parsed);
        }
    }
    best
}

fn labeled_token_has_alnum_prefix(agent_text: &str, start: usize) -> bool {
    if start == 0 {
        return false;
    }
    agent_text[..start]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn parse_u8_in_range_1_10(value: &str) -> Option<u8> {
    let digits: String = value.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &value[digits.len()..];
    // Reject range templates such as "1-10" or "1/10".
    if rest.starts_with('-') || rest.starts_with('/') {
        return None;
    }
    let score: u8 = digits.parse().ok()?;
    (1..=10).contains(&score).then_some(score)
}

fn parse_yes_no_token(value: &str) -> Option<bool> {
    if let Some(rest) = value.strip_prefix("YES") {
        if token_boundary_after_label(rest) {
            return Some(true);
        }
    }
    if let Some(rest) = value.strip_prefix("NO") {
        if token_boundary_after_label(rest) {
            return Some(false);
        }
    }
    None
}

fn token_boundary_after_label(rest: &str) -> bool {
    !rest.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
}

/// # Errors
/// Returns an error when `COMPLEXITY_SCORE` is missing, malformed, or outside 1–10.
pub(crate) fn parse_complexity_score(agent_text: &str) -> Result<u8, String> {
    last_labeled_value(agent_text, "COMPLEXITY_SCORE", parse_u8_in_range_1_10).ok_or_else(|| {
        "router_a_1: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })
}

/// # Errors
/// Returns an error when `CODING_TASK` is missing or not `YES`/`NO`.
pub(crate) fn parse_coding_task(agent_text: &str) -> Result<bool, String> {
    last_labeled_value(agent_text, "CODING_TASK", parse_yes_no_token).ok_or_else(|| {
        "router_a_2: missing or malformed CODING_TASK in agent response".to_string()
    })
}

#[cfg(test)]
mod router_wants_continue_tests {
    use super::router_wants_continue;

    #[test]
    fn continue_marker_forms() {
        assert!(router_wants_continue("CONTINUE_ROUTER"));
        assert!(router_wants_continue("CONTINUE_ROUTER\n\n"));
        assert!(router_wants_continue("CONTINUE_ROUTER\n"));
        assert!(!router_wants_continue(
            "Summary\n\nEvidence shows the fix works.\n"
        ));
        assert!(!router_wants_continue(
            "Please output CONTINUE_ROUTER when done."
        ));
    }
}

#[cfg(test)]
mod trivial_social_request_tests {
    use super::is_trivial_social_request;

    #[test]
    fn social_vs_coding_requests() {
        assert!(is_trivial_social_request("Hello"));
        assert!(is_trivial_social_request("hello!"));
        assert!(is_trivial_social_request("  hi  "));
        assert!(is_trivial_social_request("thanks"));
        assert!(is_trivial_social_request("Thank you"));
        assert!(is_trivial_social_request("good morning"));
        assert!(!is_trivial_social_request("fix bug"));
        assert!(!is_trivial_social_request("Fix the bug"));
        assert!(!is_trivial_social_request("hi, fix src/foo.rs"));
        assert!(!is_trivial_social_request("Hello world program"));
        assert!(!is_trivial_social_request("thanks for reviewing PR #12"));
        assert!(!is_trivial_social_request("implement the feature"));
        assert!(!is_trivial_social_request(""));
        assert!(!is_trivial_social_request("   "));
        assert!(!is_trivial_social_request(&"hello ".repeat(20)));
    }
}

#[cfg(test)]
mod parse_router_a_tokens_tests {
    use super::{
        labeled_token_has_alnum_prefix, last_labeled_value, parse_coding_task,
        parse_complexity_score, parse_u8_in_range_1_10, parse_yes_no_token,
        token_boundary_after_label,
    };

    #[test]
    fn parse_complexity_and_coding_flexible_forms() {
        assert_eq!(parse_complexity_score("analysis\nCOMPLEXITY_SCORE: 3\n").unwrap(), 3);
        assert_eq!(
            parse_complexity_score("done\n```\nCOMPLEXITY_SCORE: 7\n```\n").unwrap(),
            7
        );
        // Session 20260712_173852_20umca3f: mid-line glue, digits stuck to prose.
        assert_eq!(
            parse_complexity_score("score.COMPLEXITY_SCORE: 4I'll check").unwrap(),
            4
        );
        assert_eq!(parse_complexity_score("**COMPLEXITY_SCORE** : `5`").unwrap(), 5);
        assert_eq!(
            parse_complexity_score("COMPLEXITY_SCORE: 1-10\nCOMPLEXITY_SCORE: 2\n").unwrap(),
            2
        );
        assert!(parse_u8_in_range_1_10("1-10").is_none());
        assert!(parse_complexity_score("COMPLEXITY_SCORE: 11\n").is_err());
        assert!(parse_complexity_score("COMPLEXITY_SCORE: 0\n").is_err());
        assert!(parse_complexity_score("no score\n").is_err());
        assert!(parse_complexity_score("COMPLEEXITY_SCORE: 1\n").is_err());
        assert!(parse_coding_task("CODING_TASK: YES\n").unwrap());
        assert!(!parse_coding_task("CODING_TASK: NO\n").unwrap());
        assert!(parse_coding_task("done.CODING_TASK: YES'll").unwrap());
        assert!(!parse_coding_task("`CODING_TASK`: **NO**").unwrap());
        assert!(parse_coding_task("CODING_TASK: YES\nCODING_TASK: NO\n").is_ok_and(|v| !v));
        assert!(parse_coding_task("CODING_TASK: NOTHING\n").is_err());
        assert!(parse_coding_task("CODING_TASK: maybe\n").is_err());
        assert!(parse_coding_task("no token\n").is_err());
    }

    #[test]
    fn score_helpers_named_for_kiss_coverage() {
        assert_eq!(parse_u8_in_range_1_10("8x"), Some(8));
        assert_eq!(parse_yes_no_token("YES"), Some(true));
        assert_eq!(parse_yes_no_token("NO"), Some(false));
        assert!(parse_yes_no_token("maybe").is_none());
        assert!(token_boundary_after_label(""));
        assert!(!token_boundary_after_label("THING"));
        assert!(!labeled_token_has_alnum_prefix("COMPLEXITY_SCORE", 0));
        assert!(labeled_token_has_alnum_prefix("XCOMPLEXITY_SCORE", 1));
        assert_eq!(
            last_labeled_value(".COMPLEXITY_SCORE: 9", "COMPLEXITY_SCORE", parse_u8_in_range_1_10),
            Some(9)
        );
    }
}
