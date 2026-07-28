//! Parse `KPop` chat headings for default-route residual work / stop signal.

/// Returns true when every group index in `1..=group_count` has a
/// `NO_WORK_REMAINING N` heading-like marker and no `Group Work N` marker.
///
/// Accepts canonical whole-line `## TOKEN N`, mid-line glued forms (for example
/// `…untouched.## NO_WORK_REMAINING 1`), optional `#`…`######` depth, and bare
/// `TOKEN N` when the token is not mid-identifier. Wrong-index and ambiguous
/// forms still fail.
///
/// Missing either marker for an index, or both markers for an index, means work remains
/// (`false`). `group_count == 0` is never all-no-work (caller should not reach here with
/// empty groups after validation).
#[must_use]
pub(crate) fn all_groups_no_work_remaining(chat: &str, group_count: usize) -> bool {
    if group_count == 0 {
        return false;
    }
    for n in 1..=group_count {
        let has_no_work = chat_has_indexed_heading(chat, "NO_WORK_REMAINING", n);
        let has_group_work = chat_has_indexed_heading(chat, "Group Work", n);
        if !has_no_work || has_group_work {
            return false;
        }
    }
    true
}

fn chat_has_indexed_heading(chat: &str, token: &str, n: usize) -> bool {
    let mut from = 0;
    while let Some(rel) = chat[from..].find(token) {
        let token_start = from + rel;
        if indexed_heading_at(chat, token_start, token, n) {
            return true;
        }
        from = token_start + token.len();
    }
    false
}

/// `token` begins at `token_start`; require a heading-like prefix and index `n`.
fn indexed_heading_at(chat: &str, token_start: usize, token: &str, n: usize) -> bool {
    if !heading_prefix_ok(&chat[..token_start]) {
        return false;
    }
    let after_token = &chat[token_start + token.len()..];
    // Optional separators before the index (whitespace, em/en dash, hyphen, colon).
    let rest = after_token.trim_start_matches(|c: char| {
        c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':')
    });
    let digits = format!("{n}");
    if !rest.starts_with(&digits) {
        return false;
    }
    let after = &rest[digits.len()..];
    after.is_empty()
        || after.starts_with(|c: char| {
            c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':' | '*' | '`')
        })
}

/// Prefix before the token: optional same-line markdown `#`…`######`, optional
/// spaces/tabs, and a non-identifier boundary (start of chat, newline, or punctuation).
fn heading_prefix_ok(before: &str) -> bool {
    let mut chars: Vec<char> = before.chars().collect();
    while matches!(chars.last().copied(), Some(' ' | '\t')) {
        chars.pop();
    }
    let mut hash_count = 0usize;
    while matches!(chars.last().copied(), Some('#')) {
        hash_count += 1;
        if hash_count > 6 {
            return false;
        }
        chars.pop();
    }
    chars
        .last()
        .copied()
        .is_none_or(|c| c == '\n' || c == '\r' || !(c.is_ascii_alphanumeric() || c == '_'))
}

#[cfg(test)]
mod tests {
    use super::{all_groups_no_work_remaining, chat_has_indexed_heading};

    #[test]
    fn all_no_work_when_every_index_has_no_work_only() {
        let chat = "## NO_WORK_REMAINING 1\n## NO_WORK_REMAINING 2 — done\n";
        assert!(all_groups_no_work_remaining(chat, 2));
    }

    #[test]
    fn work_remaining_when_any_group_work() {
        let chat = "## NO_WORK_REMAINING 1\n## Group Work 2\n- do stuff\n";
        assert!(!all_groups_no_work_remaining(chat, 2));
    }

    #[test]
    fn work_remaining_when_both_markers_for_same_index() {
        let chat = "## NO_WORK_REMAINING 1\n## Group Work 1\nplan\n";
        assert!(!all_groups_no_work_remaining(chat, 1));
    }

    #[test]
    fn work_remaining_when_marker_missing() {
        let chat = "## NO_WORK_REMAINING 1\n";
        assert!(!all_groups_no_work_remaining(chat, 2));
        assert!(!all_groups_no_work_remaining("", 1));
    }

    #[test]
    fn accepts_optional_separators_around_index() {
        assert!(chat_has_indexed_heading(
            "## NO_WORK_REMAINING — 3 : trailing",
            "NO_WORK_REMAINING",
            3
        ));
        assert!(chat_has_indexed_heading("## Group Work:2", "Group Work", 2));
        assert!(!chat_has_indexed_heading(
            "## NO_WORK_REMAINING 12",
            "NO_WORK_REMAINING",
            1
        ));
    }

    #[test]
    fn zero_groups_is_not_all_no_work() {
        assert!(!all_groups_no_work_remaining("## NO_WORK_REMAINING 1", 0));
    }

    #[test]
    fn accepts_mid_line_glued_hash_heading() {
        let chat = "workspace stayed untouched.## NO_WORK_REMAINING 1 — greeting already delivered; workspace untouched\n\n**tl;dr:** done.";
        assert!(all_groups_no_work_remaining(chat, 1));
        assert!(chat_has_indexed_heading(
            "already delivered.## Group Work 1\nDo the greeting.",
            "Group Work",
            1
        ));
    }

    #[test]
    fn accepts_bare_and_alternate_hash_depth() {
        assert!(chat_has_indexed_heading(
            "NO_WORK_REMAINING 1 — done",
            "NO_WORK_REMAINING",
            1
        ));
        assert!(chat_has_indexed_heading(
            "prose above\nNO_WORK_REMAINING 2",
            "NO_WORK_REMAINING",
            2
        ));
        assert!(chat_has_indexed_heading(
            "# NO_WORK_REMAINING 2",
            "NO_WORK_REMAINING",
            2
        ));
        assert!(chat_has_indexed_heading(
            "### Group Work 1\nplan",
            "Group Work",
            1
        ));
        assert!(all_groups_no_work_remaining("NO_WORK_REMAINING 1\n", 1));
    }

    #[test]
    fn rejects_mid_identifier_and_wrong_index_near_misses() {
        assert!(!chat_has_indexed_heading(
            "emitNO_WORK_REMAINING 1",
            "NO_WORK_REMAINING",
            1
        ));
        assert!(!chat_has_indexed_heading(
            "prefixNO_WORK_REMAINING 1",
            "NO_WORK_REMAINING",
            1
        ));
        assert!(!chat_has_indexed_heading(
            "emit NO_WORK_REMAINING 1",
            "NO_WORK_REMAINING",
            1
        ));
        assert!(!chat_has_indexed_heading(
            "## NO_WORK_REMAINING 12",
            "NO_WORK_REMAINING",
            1
        ));
        assert!(!all_groups_no_work_remaining(
            "still considering NO_WORK later",
            1
        ));
    }
}
