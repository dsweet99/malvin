use std::io::{self, Write};

use super::tag_list::{list_line, write_body, write_list};

struct AdviceEntry {
    tag: &'static str,
    body: &'static str,
    description: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/advice_registry.rs"));

const LEGACY_ADVICE_ALIASES: &[(&str, &str)] = &[("doc_design", "prin_doc")];

fn canonical_advice_tag(tag: &str) -> &str {
    LEGACY_ADVICE_ALIASES
        .iter()
        .find_map(|(legacy, canonical)| (*legacy == tag).then_some(*canonical))
        .unwrap_or(tag)
}

fn advice_entry(tag: &str) -> Result<&'static AdviceEntry, String> {
    if !is_valid_advice_tag(tag) {
        return Err(format!(
            "invalid --advice TAG `{tag}`: must be lowercase letters/digits/underscores, start with a letter, at most 32 characters"
        ));
    }
    let canonical = canonical_advice_tag(tag);
    ADVICE_ENTRIES
        .iter()
        .find(|e| e.tag == canonical)
        .ok_or_else(|| {
            let mut known: Vec<&str> = ADVICE_ENTRIES.iter().map(|e| e.tag).collect();
            known.extend(LEGACY_ADVICE_ALIASES.iter().map(|(legacy, _)| *legacy));
            known.sort_unstable();
            format!("unknown --advice TAG `{tag}` (known: {})", known.join(", "))
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

#[cfg(test)]
fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

pub(crate) fn print_advice_list_to_writer(out: impl Write) -> Result<(), String> {
    let lines: Vec<String> = ADVICE_ENTRIES
        .iter()
        .map(|e| list_line(e.tag, e.description))
        .collect();
    write_list(out, "--advice", &lines)
}

pub(crate) fn print_advice_list() -> Result<(), String> {
    print_advice_list_to_writer(io::stdout().lock())
}

pub(crate) fn print_advice_to_writer(tag: &str, out: impl Write) -> Result<(), String> {
    write_body(out, advice_text(tag)?)
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

    fn doc_design_tag_prints_document_design() {
        let mut buf = Vec::new();
        print_advice_to_writer("doc_design", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("Design for the Reader"));
        assert!(s.contains("Make Structure Visible"));
    }

    fn scholar_tag_prints_scholarly_advice() {
        let mut buf = Vec::new();
        print_advice_to_writer("scholar", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("XYZ+1"));
        assert!(s.contains("Canonical structure"));
    }

    fn report_tag_prints_report_advice() {
        let mut buf = Vec::new();
        print_advice_to_writer("report", &mut buf).expect("print");
        let s = String::from_utf8(buf).expect("utf8");
        assert!(s.contains("concrete evidence"));
        assert!(s.contains("complete sentences"));
    }

    fn unknown_tag_errors() {
        let err = advice_text("nope").expect_err("unknown");
        assert!(err.contains("unknown"), "{err}");
        assert!(err.contains("doc_design"), "{err}");
        assert!(err.contains("report"), "{err}");
        assert!(err.contains("scholar"), "{err}");
        assert!(!err.contains("facts"), "{err}");
    }

    fn invalid_tag_shape_errors() {
        assert!(advice_text("Bad").is_err());
        assert!(advice_text("1abc").is_err());
        assert!(advice_text("").is_err());
        assert!(advice_text("a-b").is_err());
        assert!(advice_text(&"a".repeat(33)).is_err());
    }

    fn valid_tag_shape_accepted_for_lookup() {
        assert!(is_valid_advice_tag("design"));
        assert!(is_valid_advice_tag("doc_design"));
        assert!(is_valid_advice_tag("a1"));
        assert!(is_valid_advice_tag("abcdefg"));
        assert!(is_valid_advice_tag(&"a".repeat(32)));
    }

    fn advice_list_prints_tag_colon_description_lines() {
        let mut buf = Vec::new();
        print_advice_list_to_writer(&mut buf).expect("list");
        let s = String::from_utf8(buf).expect("utf8");
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(
            lines[0], "Type `malvin --advice TAG` to see a full document.",
            "{s}"
        );
        assert_eq!(lines[1], "", "{s}");
        assert_eq!(lines[2], "TAG: Description", "{s}");
        assert!(!s.contains("List format"), "{s}");
        assert!(s.ends_with("\n\n"), "{s}");
        let expected: Vec<String> = ADVICE_ENTRIES
            .iter()
            .map(|e| format!("{}: {}", e.tag, e.description))
            .collect();
        assert_eq!(lines[3..lines.len() - 1], expected[..], "{s}");
        assert_eq!(lines.last(), Some(&""), "{s}");
    }

    fn all_advice_descriptions_are_within_word_limit() {
        for entry in ADVICE_ENTRIES {
            let Some((base, _)) = entry.description.split_once(" [") else {
                panic!("{} missing size suffix", entry.tag);
            };
            let n = word_count(base);
            assert!(
                n <= crate::desc_limit::MAX_DESC_WORDS,
                "tag `{}` description has {n} words: {base}",
                entry.tag
            );
        }
    }

    fn advice_descriptions_come_from_first_line_without_prefix() {
        for entry in ADVICE_ENTRIES {
            let content = entry.body;
            let first = content.lines().next().expect("non-empty advice md");
            let value = first
                .strip_prefix("description:")
                .unwrap_or_else(|| panic!("{}: first line must be `description: ...`", entry.tag))
                .trim();
            let lines = content.lines().count();
            let words = content.split_whitespace().count();
            let line_word = if lines == 1 { "line" } else { "lines" };
            let expected = format!("{value} [{lines} {line_word}, {words} words]");
            assert_eq!(
                entry.description, expected,
                "registry description must match first-line value plus embedded file size for {}",
                entry.tag
            );
            assert!(
                !entry.description.starts_with("description:"),
                "description must not retain the `description:` prefix: {}",
                entry.description
            );
        }
    }

    fn every_advice_md_stem_is_registered_as_tag() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/prompts/advice");
        let mut stems = HashSet::new();
        for entry in fs::read_dir(&dir).expect("read advice dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if !crate::advice_registry_build::is_advice_markdown(&path) {
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

    #[test]
    fn merged_doc_design_tag_prints_document_design() {
        doc_design_tag_prints_document_design();
        scholar_tag_prints_scholarly_advice();
        report_tag_prints_report_advice();
        unknown_tag_errors();
        invalid_tag_shape_errors();
        valid_tag_shape_accepted_for_lookup();
        advice_list_prints_tag_colon_description_lines();
        all_advice_descriptions_are_within_word_limit();
        advice_descriptions_come_from_first_line_without_prefix();
        every_advice_md_stem_is_registered_as_tag();
    }
}
