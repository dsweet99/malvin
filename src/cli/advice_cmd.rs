use std::io::{self, Write};

const ADVICE_ENTRIES: &[(&str, &str)] = &[(
    "design",
    include_str!("../../default_prompts/advice/document_design.md"),
)];

pub(crate) fn advice_text(tag: &str) -> Result<&'static str, String> {
    if !is_valid_advice_tag(tag) {
        return Err(format!(
            "invalid --advice TAG `{tag}`: must be lowercase letters/digits, start with a letter, at most 7 characters"
        ));
    }
    ADVICE_ENTRIES
        .iter()
        .find(|(t, _)| *t == tag)
        .map(|(_, body)| *body)
        .ok_or_else(|| {
            let known: Vec<&str> = ADVICE_ENTRIES.iter().map(|(t, _)| *t).collect();
            format!(
                "unknown --advice TAG `{tag}` (known: {})",
                known.join(", ")
            )
        })
}

fn is_valid_advice_tag(tag: &str) -> bool {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() || tag.len() > 7 {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

pub(crate) fn print_advice_to_writer(tag: &str, mut out: impl Write) -> Result<(), String> {
    let text = advice_text(tag)?;
    out.write_all(text.as_bytes())
        .map_err(|e| format!("stdout: {e}"))?;
    if !text.ends_with('\n') {
        out.write_all(b"\n").map_err(|e| format!("stdout: {e}"))?;
    }
    Ok(())
}

pub(crate) fn print_advice(tag: &str) -> Result<(), String> {
    print_advice_to_writer(tag, io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_tag_prints_document_design() {
        let mut buf = Vec::new();
        print_advice_to_writer("design", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("C.R.A.P."));
        assert!(s.contains("Contrast"));
    }

    #[test]
    fn unknown_tag_errors() {
        let err = advice_text("nope").expect_err("unknown");
        assert!(err.contains("unknown"), "{err}");
        assert!(err.contains("design"), "{err}");
    }

    #[test]
    fn invalid_tag_shape_errors() {
        assert!(advice_text("Bad").is_err());
        assert!(advice_text("1abc").is_err());
        assert!(advice_text("toolongx").is_err());
        assert!(advice_text("").is_err());
        assert!(advice_text("a-b").is_err());
    }

    #[test]
    fn valid_tag_shape_accepted_for_lookup() {
        assert!(is_valid_advice_tag("design"));
        assert!(is_valid_advice_tag("a1"));
        assert!(is_valid_advice_tag("abcdefg"));
    }
}
