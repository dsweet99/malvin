//! Extract ```bash … ``` fenced blocks from assistant text.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BashFence {
    pub command: String,
}

/// Returns bash fence commands in document order.
#[must_use]
pub fn parse_bash_fences(text: &str) -> Vec<BashFence> {
    let mut out = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if !is_bash_fence_open(trimmed) {
            continue;
        }
        let mut cmd = String::new();
        for inner in lines.by_ref() {
            if inner.trim() == "```" {
                break;
            }
            if !cmd.is_empty() {
                cmd.push('\n');
            }
            cmd.push_str(inner);
        }
        if !cmd.trim().is_empty() {
            out.push(BashFence { command: cmd });
        }
    }
    out
}

fn is_bash_fence_open(trimmed: &str) -> bool {
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

#[cfg(test)]
#[path = "fence_parser_test.rs"]
mod fence_parser_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<BashFence> = None;
    }
}
