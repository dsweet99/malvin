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
        "router_a: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })?;
    let score: u32 = value.parse().map_err(|_| {
        "router_a: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })?;
    if !(1..=10).contains(&score) {
        return Err("router_a: missing or malformed COMPLEXITY_SCORE in agent response".to_string());
    }
    u8::try_from(score).map_err(|_| {
        "router_a: missing or malformed COMPLEXITY_SCORE in agent response".to_string()
    })
}

/// # Errors
///
/// Returns an error when `CODING_TASK` is missing or not `YES`/`NO`.
pub(crate) fn parse_coding_task(agent_text: &str) -> Result<bool, String> {
    let value = parse_scored_line(agent_text, "CODING_TASK:")
        .ok_or_else(|| "router_a: missing or malformed CODING_TASK in agent response".to_string())?;
    match value.as_str() {
        "YES" => Ok(true),
        "NO" => Ok(false),
        _ => Err("router_a: missing or malformed CODING_TASK in agent response".to_string()),
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
