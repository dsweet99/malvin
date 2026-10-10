use std::io::{self, Write};

pub(crate) const CREDITS_DOC: &str = include_str!("../../default_prompts/credits.md");

pub(crate) fn print_credits_to_writer(mut out: impl Write) -> Result<(), String> {
    out.write_all(CREDITS_DOC.as_bytes())
        .map_err(|e| format!("stdout: {e}"))
}

pub(crate) fn print_credits() -> Result<(), String> {
    print_credits_to_writer(io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_name_boden_and_foerster() {
        let mut buf = Vec::new();
        print_credits_to_writer(&mut buf).expect("print");
        let text = String::from_utf8(buf).expect("utf8");
        assert!(!text.contains("Karl Popper"));
        assert!(text.contains("Margaret Boden"));
        assert!(text.contains("Exploratory Creativity"));
        assert!(text.contains("The Creative Mind: Myths and Mechanisms"));
        assert!(text.contains("How to ML Paper"));
        assert!(text.contains("Jakob N. Foerster"));
    }

    #[test]
    fn print_credits_to_stdout_succeeds() {
        print_credits().expect("print");
    }
}
