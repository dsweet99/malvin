use std::io::{self, Write};

struct AdviceEntry {
    tag: &'static str,
    body: &'static str,
    description: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/advice_registry.rs"));

fn advice_entry(tag: &str) -> Result<&'static AdviceEntry, String> {
    if !is_valid_advice_tag(tag) {
        return Err(format!(
            "invalid --advice TAG `{tag}`: must be lowercase letters/digits/underscores, start with a letter, at most 32 characters"
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
    if !first.is_ascii_lowercase() || tag.len() > 32 {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
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
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    #[test]
    fn doc_design_tag_prints_document_design() {
        let mut buf = Vec::new();
        print_advice_to_writer("doc_design", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("C.R.A.P."));
        assert!(s.contains("Contrast"));
    }

    #[test]
    fn scholar_tag_prints_scholarly_advice() {
        let mut buf = Vec::new();
        print_advice_to_writer("scholar", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("XYZ+1"));
        assert!(s.contains("Canonical structure"));
    }

    #[test]
    fn report_tag_prints_report_advice() {
        let mut buf = Vec::new();
        print_advice_to_writer("report", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("technically sophisticated reader"));
        assert!(s.contains("plain English"));
    }

    #[test]
    fn unknown_tag_errors() {
        let err = advice_text("nope").expect_err("unknown");
        assert!(err.contains("unknown"), "{err}");
        assert!(err.contains("doc_design"), "{err}");
        assert!(err.contains("report"), "{err}");
        assert!(err.contains("scholar"), "{err}");
    }

    #[test]
    fn invalid_tag_shape_errors() {
        assert!(advice_text("Bad").is_err());
        assert!(advice_text("1abc").is_err());
        assert!(advice_text("").is_err());
        assert!(advice_text("a-b").is_err());
        assert!(advice_text(&"a".repeat(33)).is_err());
    }

    #[test]
    fn valid_tag_shape_accepted_for_lookup() {
        assert!(is_valid_advice_tag("design"));
        assert!(is_valid_advice_tag("doc_design"));
        assert!(is_valid_advice_tag("a1"));
        assert!(is_valid_advice_tag("abcdefg"));
        assert!(is_valid_advice_tag(&"a".repeat(32)));
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
        for entry in ADVICE_ENTRIES {
            assert!(
                s.contains(&format!("{}\t{}", entry.tag, entry.description)),
                "expected tab after tag for {}, got: {s}",
                entry.tag
            );
            assert!(word_count(entry.description) <= 7);
            assert!(
                s.contains(&format!("{} lines", advice_line_count(entry.body))),
                "{s}"
            );
            assert!(
                s.contains(&format!("{} characters", advice_char_count(entry.body))),
                "{s}"
            );
        }
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

    #[test]
    fn advice_descriptions_come_from_first_line_without_prefix() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("default_prompts/advice");
        for entry in ADVICE_ENTRIES {
            let path = dir.join(format!("{}.md", entry.tag));
            let content = fs::read_to_string(&path).expect("read advice md");
            let first = content.lines().next().expect("non-empty advice md");
            let value = first
                .strip_prefix("description:")
                .unwrap_or_else(|| panic!("{}: first line must be `description: ...`", entry.tag))
                .trim();
            assert_eq!(
                entry.description, value,
                "registry description must match first-line value for {}",
                entry.tag
            );
            assert!(
                !entry.description.starts_with("description:"),
                "description must not retain the `description:` prefix: {}",
                entry.description
            );
        }
    }

    #[test]
    fn every_advice_md_stem_is_registered_as_tag() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("default_prompts/advice");
        let mut stems = HashSet::new();
        for entry in fs::read_dir(&dir).expect("read advice dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("utf8 stem")
                .to_string();
            assert!(
                is_valid_advice_tag(&stem),
                "advice file stem `{stem}` must be a valid --advice TAG"
            );
            stems.insert(stem);
        }
        let tags: HashSet<&str> = ADVICE_ENTRIES.iter().map(|e| e.tag).collect();
        assert_eq!(
            stems,
            tags.iter().map(|t| (*t).to_string()).collect(),
            "ADVICE_ENTRIES tags must equal advice/*.md stems"
        );
        for entry in ADVICE_ENTRIES {
            assert!(stems.contains(entry.tag));
        }
    }
}
