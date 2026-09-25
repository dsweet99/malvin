use std::io::{self, Write};

struct AdviceEntry {
    tag: &'static str,
    body: &'static str,
    description: &'static str,
}

const ADVICE_ENTRIES: &[AdviceEntry] = &[AdviceEntry {
    tag: "design",
    body: include_str!("../../default_prompts/advice/document_design.md"),
    description: "Document design via C.R.A.P. principles",
}];

fn advice_entry(tag: &str) -> Result<&'static AdviceEntry, String> {
    if !is_valid_advice_tag(tag) {
        return Err(format!(
            "invalid --advice TAG `{tag}`: must be lowercase letters/digits, start with a letter, at most 7 characters"
        ));
    }
    ADVICE_ENTRIES.iter().find(|e| e.tag == tag).ok_or_else(|| {
        let known: Vec<&str> = ADVICE_ENTRIES.iter().map(|e| e.tag).collect();
        format!(
            "unknown --advice TAG `{tag}` (known: {})",
            known.join(", ")
        )
    })
}

pub(crate) fn advice_text(tag: &str) -> Result<&'static str, String> {
    advice_entry(tag).map(|e| e.body)
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

fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

fn advice_line_count(body: &str) -> usize {
    if body.is_empty() {
        return 0;
    }
    body.lines().count()
}

fn advice_char_count(body: &str) -> usize {
    body.chars().count()
}

fn format_advice_list_line(entry: &AdviceEntry) -> String {
    format!(
        "{}\t{}  {} lines, {} characters",
        entry.tag,
        entry.description,
        advice_line_count(entry.body),
        advice_char_count(entry.body)
    )
}

const ADVICE_LIST_USAGE: &str = "\
Type `malvin --advice TAG` to see a full document.
List format: tag, then description (tab after the tag), then line and character counts.";

pub(crate) fn print_advice_list_to_writer(mut out: impl Write) -> Result<(), String> {
    out.write_all(ADVICE_LIST_USAGE.as_bytes())
        .map_err(|e| format!("stdout: {e}"))?;
    out.write_all(b"\n").map_err(|e| format!("stdout: {e}"))?;
    for entry in ADVICE_ENTRIES {
        debug_assert!(
            word_count(entry.description) <= 7,
            "advice description for `{}` exceeds 7 words",
            entry.tag
        );
        let line = format_advice_list_line(entry);
        out.write_all(line.as_bytes())
            .map_err(|e| format!("stdout: {e}"))?;
        out.write_all(b"\n").map_err(|e| format!("stdout: {e}"))?;
    }
    Ok(())
}

pub(crate) fn print_advice_list() -> Result<(), String> {
    print_advice_list_to_writer(io::stdout().lock())
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

    #[test]
    fn advice_list_includes_tag_description_lines_and_chars() {
        let mut buf = Vec::new();
        print_advice_list_to_writer(&mut buf).expect("list");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(
            s.contains("Type `malvin --advice TAG` to see a full document."),
            "{s}"
        );
        assert!(s.contains("tag, then description"), "{s}");
        assert!(s.contains("design"), "{s}");
        assert!(s.contains("Document design via C.R.A.P. principles"), "{s}");
        assert!(
            s.contains("design\tDocument design via C.R.A.P. principles"),
            "expected tab after tag, got: {s}"
        );
        let design = ADVICE_ENTRIES.iter().find(|e| e.tag == "design").unwrap();
        assert!(word_count(design.description) <= 7);
        assert!(
            s.contains(&format!("{} lines", advice_line_count(design.body))),
            "{s}"
        );
        assert!(
            s.contains(&format!("{} characters", advice_char_count(design.body))),
            "{s}"
        );
    }

    #[test]
    fn all_advice_descriptions_are_at_most_seven_words() {
        for entry in ADVICE_ENTRIES {
            let n = word_count(entry.description);
            assert!(
                n <= 7,
                "tag `{}` description has {n} words: {}",
                entry.tag,
                entry.description
            );
        }
    }
}
