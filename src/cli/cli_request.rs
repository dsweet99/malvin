//! CLI positional request validation (`<REQUEST>` text or path).

pub(crate) fn require_cli_request(
    request: Option<&String>,
    subcommand: &str,
) -> Result<String, String> {
    let trimmed = request
        .map(String::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty());
    trimmed
        .map(std::string::ToString::to_string)
        .ok_or_else(|| format!("malvin {subcommand}: missing required REQUEST (text or path)"))
}

#[cfg(test)]
mod tests {
    use super::require_cli_request;

    #[test]
    fn errors_when_missing() {
        let err = require_cli_request(None, "kpop").unwrap_err();
        assert!(err.contains("kpop") && err.contains("REQUEST"));
    }

    #[test]
    fn rejects_whitespace_only_request() {
        let whitespace = Some("   ".to_string());
        let err = require_cli_request(whitespace.as_ref(), "kpop").unwrap_err();
        assert!(err.contains("kpop") && err.contains("REQUEST"));
    }

    #[test]
    fn accepts_trimmed_non_empty_request() {
        let got = require_cli_request(Some(&"  hi  ".to_string()), "do").expect("ok");
        assert_eq!(got, "hi");
    }
}
