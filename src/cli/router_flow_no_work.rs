
pub(crate) fn chat_has_malvin_done(chat: &str) -> bool {
    chat.lines().any(|line| line.trim() == "__MALVIN_DONE__")
}

#[cfg(test)]
mod tests {
    use super::chat_has_malvin_done;

    #[test]
    fn detects_done_alone_on_a_line() {
        assert!(chat_has_malvin_done("__MALVIN_DONE__\n"));
        assert!(chat_has_malvin_done("preamble\n  __MALVIN_DONE__  \nmore\n"));
        assert!(!chat_has_malvin_done("not done yet\n"));
        assert!(!chat_has_malvin_done("__MALVIN_DONE__ trailing\n"));
        assert!(!chat_has_malvin_done("prefix __MALVIN_DONE__\n"));
    }
}
