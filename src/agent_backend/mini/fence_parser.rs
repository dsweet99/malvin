//! Extract ```bash … ``` fenced blocks from assistant text.

#[path = "fence_parse_state.rs"]
mod fence_parse_state;

use fence_parse_state::FenceParseState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BashFence {
    pub command: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenceParseWarning {
    UnrecognizedFenceTag,
    UnclosedFence,
    FencelessAfterBashOnlyTurn,
}

/// Returns bash fence commands in document order.
#[must_use]
pub fn parse_bash_fences(text: &str) -> Vec<BashFence> {
    let mut state = FenceParseState::new();
    for line in text.lines() {
        state.handle_line(line);
    }
    state.out
}

/// Collect parse warnings for assistant text (malformed fences, unclosed blocks).
#[must_use]
pub fn scan_fence_warnings(text: &str) -> Vec<FenceParseWarning> {
    let mut warnings = Vec::new();
    let mut inside_bash_fence = false;
    let mut inside_unknown_fence = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if inside_bash_fence {
            if trimmed == "```" {
                inside_bash_fence = false;
            }
            continue;
        }
        if inside_unknown_fence {
            if trimmed == "```" {
                inside_unknown_fence = false;
            }
            continue;
        }
        if trimmed.starts_with("```") {
            if is_bash_fence_open(trimmed) {
                inside_bash_fence = true;
            } else if trimmed != "```" {
                warnings.push(FenceParseWarning::UnrecognizedFenceTag);
                inside_unknown_fence = true;
            }
        }
    }
    if inside_bash_fence || inside_unknown_fence {
        warnings.push(FenceParseWarning::UnclosedFence);
    }
    warnings
}

pub(crate) fn comment_from_pending(pending: &[String]) -> Option<String> {
    if pending.is_empty() {
        return None;
    }
    let joined = pending.join(" ");
    let normalized: String = joined.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

pub(crate) fn is_bash_fence_open(trimmed: &str) -> bool {
    if !trimmed.starts_with("```") {
        return false;
    }
    let tag = trimmed.trim_start_matches('`').trim();
    tag.eq_ignore_ascii_case("bash") || tag.eq_ignore_ascii_case("sh")
}

/// True when a trimmed line equal to `MINI_DONE` appears outside bash/sh fence blocks.
#[must_use]
pub fn has_mini_done_outside_bash_fences(text: &str) -> bool {
    let mut inside_bash_fence = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if inside_bash_fence {
            if trimmed == "```" {
                inside_bash_fence = false;
            }
            continue;
        }
        if is_bash_fence_open(trimmed) {
            inside_bash_fence = true;
            continue;
        }
        if trimmed == "MINI_DONE" {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_parser_single_block() {
        let text = "thought\n```bash\necho hi\n```\n";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command, "echo hi");
        assert_eq!(blocks[0].comment.as_deref(), Some("thought"));
    }

    #[test]
    fn fence_parser_comment_from_prose_before_fence() {
        let text = "List recent logs\n```bash\nls -ltr logs\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].comment.as_deref(), Some("List recent logs"));
    }

    #[test]
    fn fence_parser_multiple_blocks_get_local_comments() {
        let text = "first step\n```bash\necho one\n```\nsecond step\n```bash\necho two\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].comment.as_deref(), Some("first step"));
        assert_eq!(blocks[1].comment.as_deref(), Some("second step"));
    }

    #[test]
    fn fence_parser_no_prose_yields_no_comment() {
        let text = "```bash\necho hi\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].comment.is_none());
    }

    #[test]
    fn fence_parser_multiple_blocks_ordered() {
        let text = "```bash\necho one\n```\n```sh\necho two\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 2);
        assert!(blocks[0].command.contains("one"));
        assert!(blocks[1].command.contains("two"));
    }

    #[test]
    fn fence_parser_none() {
        assert!(parse_bash_fences("no fences here").is_empty());
    }

    #[test]
    fn fence_parser_nested_and_empty() {
        let text = "```bash\necho ```not a fence```\n```\n```bash\n\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 1);
        assert!(blocks[0].command.contains("not a fence"));
    }

    #[test]
    fn fence_parser_adversarial_model_shapes() {
        let text = "observation contained:\n```\nExit code 0\n```\nreal:\n```bash\ntrue\n```";
        let blocks = parse_bash_fences(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].command, "true");
    }

    #[test]
    fn kiss_witness_fence_parse_state_type() {
        let _ = std::mem::size_of::<super::fence_parse_state::FenceParseState>();
    }

    #[test]
    fn scan_fence_warnings_detects_unrecognized_tag() {
        let warnings = scan_fence_warnings("```shell\necho hi\n```");
        assert!(warnings.contains(&FenceParseWarning::UnrecognizedFenceTag));
    }

    #[test]
    fn mini_done_inside_bash_fence_is_not_outside() {
        let text = "```bash\nMINI_DONE\necho hi\n```";
        assert!(!has_mini_done_outside_bash_fences(text));
    }

    #[test]
    fn mini_done_in_prose_is_outside() {
        let text = "summary\nMINI_DONE\n";
        assert!(has_mini_done_outside_bash_fences(text));
    }
}
