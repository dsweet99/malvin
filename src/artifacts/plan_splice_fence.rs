//! Fenced-markdown extraction for Prompt 3 of `malvin plan`.

use super::PlanFileError;

/// Extract inner text from the first ```markdown or ``` fenced block in `response`.
pub fn extract_fenced_markdown_block(response: &str) -> Result<String, PlanFileError> {
    let trimmed = response.trim();
    for fence in ["```markdown", "```md", "```"] {
        if let Some(body) = extract_fence_body(trimmed, fence) {
            if !body.trim().is_empty() {
                return Ok(body.trim().to_string());
            }
        }
    }
    Err(PlanFileError::MissingFencedBlock)
}

fn extract_fence_body(text: &str, fence: &str) -> Option<String> {
    let start = text.find(fence)?;
    if fence == "```" {
        let after_marker = &text[start + fence.len()..];
        if after_marker.starts_with("markdown") || after_marker.starts_with("md") {
            return None;
        }
    }
    let mut after_open = &text[start + fence.len()..];
    if let Some(stripped) = after_open.strip_prefix('\r') {
        after_open = stripped;
    }
    after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let close = find_matching_fence_close(after_open)?;
    Some(after_open[..close].to_string())
}

/// Find the byte offset of the outer closing fence in `after_open` (content after the opener).
fn find_matching_fence_close(after_open: &str) -> Option<usize> {
    if let Some(close) = find_matching_fence_close_by_depth(after_open) {
        return Some(close);
    }
    // Inline close without a preceding newline (e.g. "```\ninline```").
    after_open.rfind("```")
}

fn find_matching_fence_close_by_depth(after_open: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut line_start = 0usize;
    while line_start < after_open.len() {
        let line_end = after_open[line_start..]
            .find('\n')
            .map_or(after_open.len(), |i| line_start + i);
        let line = &after_open[line_start..line_end];
        if line.starts_with("```") {
            if is_fence_close_line(line) {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(line_start);
                }
            } else if is_fence_open_line(line) {
                depth += 1;
            }
        }
        if line_end == after_open.len() {
            break;
        }
        line_start = line_end + 1;
    }
    None
}

fn is_fence_close_line(line: &str) -> bool {
    line.starts_with("```") && line[3..].trim().is_empty()
}

fn is_fence_open_line(line: &str) -> bool {
    line.starts_with("```") && !line[3..].trim().is_empty()
}

mod fence_fn_coverage {
    use super::*;

    #[test]
    fn extract_fence_body_skips_plain_fence_with_markdown_prefix() {
        assert!(extract_fence_body("```markdown\nx\n```", "```").is_none());
    }

    #[test]
    fn find_matching_fence_close_returns_none_for_unclosed_nested_fence() {
        assert!(find_matching_fence_close_by_depth("```bash\nx\n").is_none());
    }

    #[test]
    fn find_matching_fence_close_handles_adjacent_inner_fences() {
        let after_open = concat!(
            "```json\n{}\n```\n",
            "```bash\necho\n```\n",
            "tail\n",
            "```\n",
        );
        let close = find_matching_fence_close_by_depth(after_open).expect("close");
        assert!(after_open[..close].ends_with("tail\n"));
    }

    #[test]
    fn is_fence_line_classifiers_distinguish_open_and_close() {
        assert!(is_fence_open_line("```bash"));
        assert!(!is_fence_close_line("```bash"));
        assert!(is_fence_close_line("```"));
        assert!(!is_fence_open_line("```"));
    }

    #[test]
    fn find_matching_fence_close_falls_back_to_rfind_for_inline_close() {
        assert_eq!(find_matching_fence_close("inline```"), Some(6));
    }

    #[test]
    fn extract_fenced_markdown_block_accepts_markdown_and_md_fences() {
        assert_eq!(
            extract_fenced_markdown_block("```markdown\nhello\n```").expect("markdown"),
            "hello"
        );
        assert_eq!(
            extract_fenced_markdown_block("```md\nworld\n```").expect("md"),
            "world"
        );
        assert_eq!(
            extract_fenced_markdown_block("```\nplain\n```").expect("plain"),
            "plain"
        );
    }

    #[test]
    fn extract_fenced_markdown_block_rejects_empty_or_missing_fence() {
        assert!(extract_fenced_markdown_block("```markdown\n\n```").is_err());
        assert!(extract_fenced_markdown_block("no fence here").is_err());
    }
}
