use malvin::output::{MALVIN_DONE, is_sentinel_line};

pub(crate) fn chat_has_malvin_done(chat: &str) -> bool {
    let lines: Vec<&str> = chat.lines().collect();
    let Some(last) = lines.iter().rposition(|line| !line.trim_ascii().is_empty()) else {
        return false;
    };
    let mut fence = malvin::output::CodeFence::new();
    for line in &lines[..last] {
        fence.observe(line);
    }
    !fence.is_open() && is_sentinel_line(lines[last], MALVIN_DONE)
}

#[cfg(test)]
mod tests {
    use super::chat_has_malvin_done;

    #[test]
    fn detects_done_on_the_last_non_blank_line() {
        assert!(chat_has_malvin_done("__MALVIN_DONE__\n"));
        assert!(chat_has_malvin_done("preamble\n  __MALVIN_DONE__  \n\n"));
        assert!(chat_has_malvin_done(
            "```\ncode\n```\nDone.\n__MALVIN_DONE__"
        ));
        assert!(!chat_has_malvin_done(""));
        assert!(!chat_has_malvin_done("not done yet\n"));
        assert!(!chat_has_malvin_done("__MALVIN_DONE__ trailing\n"));
        assert!(!chat_has_malvin_done("prefix __MALVIN_DONE__\n"));
    }

    #[test]
    fn ignores_a_quoted_or_mid_reply_marker() {
        assert!(!chat_has_malvin_done(
            "No impasse. I am therefore not writing\n```\n__MALVIN_DONE__\n```\n"
        ));
        assert!(!chat_has_malvin_done("```\n__MALVIN_DONE__\n"));
        assert!(!chat_has_malvin_done("preamble\n__MALVIN_DONE__\nmore\n"));
    }
}
