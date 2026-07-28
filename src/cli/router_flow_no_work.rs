//! Parse `KPop` chat headings for default-route residual work / stop signal.

/// Returns true when every group index in `1..=group_count` has a
/// `## NO_WORK_REMAINING N` heading and no `## Group Work N` heading.
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
    chat.lines().any(|line| is_indexed_heading_line(line, token, n))
}

fn is_indexed_heading_line(line: &str, token: &str, n: usize) -> bool {
    let t = line.trim_start();
    let prefix = format!("## {token}");
    let Some(rest) = t.strip_prefix(&prefix) else {
        return false;
    };
    // Optional separators before the index (whitespace, em/en dash, hyphen, colon).
    let rest = rest.trim_start_matches(|c: char| {
        c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':')
    });
    let digits = format!("{n}");
    if !rest.starts_with(&digits) {
        return false;
    }
    let after = &rest[digits.len()..];
    after.is_empty()
        || after.starts_with(|c: char| c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':'))
}

#[cfg(test)]
mod tests {
    use super::{all_groups_no_work_remaining, is_indexed_heading_line};

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
        assert!(is_indexed_heading_line(
            "## NO_WORK_REMAINING — 3 : trailing",
            "NO_WORK_REMAINING",
            3
        ));
        assert!(is_indexed_heading_line("## Group Work:2", "Group Work", 2));
        assert!(!is_indexed_heading_line(
            "## NO_WORK_REMAINING 12",
            "NO_WORK_REMAINING",
            1
        ));
    }

    #[test]
    fn zero_groups_is_not_all_no_work() {
        assert!(!all_groups_no_work_remaining("## NO_WORK_REMAINING 1", 0));
    }
}
