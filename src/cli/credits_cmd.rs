use std::io::{self, Write};

pub(crate) const CREDITS_DOC: &str = include_str!("../../assets/prompts/docs/credits.md");

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
        assert!(text.contains("creative\n"));
        assert!(text.contains("kpop\n"));
        assert!(text.contains("scholar\n"));
        assert!(text.contains("tech_explain\n"));
        assert!(text.contains("Popper, K. (1959). The logic of scientific discovery"));
        assert!(text.contains("Bennett, M. T. (2023)."));
        assert!(text.contains("2301.12987"));
        assert!(text.contains("Boden, M. A. (2004). The creative mind: Myths and mechanisms"));
        assert!(text.contains("Foerster, J. N. (n.d.). How to ML paper"));
    }

    #[test]
    fn print_credits_to_stdout_succeeds() {
        print_credits().expect("print");
    }
}
