use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

const ADVICE_DIR: &str = "assets/prompts/advice";
const MAX_TAG_LEN: usize = 32;
const MAX_DESC_WORDS: usize = 7;

pub fn generate() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let advice_dir = manifest_dir.join(ADVICE_DIR);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", advice_dir.display());

    let mut entries = collect_entries(&advice_dir);
    entries.sort_by(|a, b| a.tag.cmp(&b.tag));
    assert_no_duplicate_tags(&entries);

    let dest = out_dir.join("advice_registry.rs");
    let code = render_registry(&entries);
    fs::write(&dest, code).unwrap_or_else(|e| panic!("write {}: {e}", dest.display()));
}

struct AdviceBuildEntry {
    tag: String,
    rel_path: String,
    description: String,
}

fn collect_entries(advice_dir: &Path) -> Vec<AdviceBuildEntry> {
    assert!(
        advice_dir.is_dir(),
        "missing advice directory {}",
        advice_dir.display()
    );
    let mut entries = Vec::new();
    let read =
        fs::read_dir(advice_dir).unwrap_or_else(|e| panic!("read {}: {e}", advice_dir.display()));
    for entry in read {
        let entry = entry.unwrap_or_else(|e| panic!("dir entry: {e}"));
        let path = entry.path();
        if !is_advice_markdown(&path) {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("non-utf8 stem: {}", path.display()));
        validate_tag(stem);
        let content =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let description = description_for(&content, stem);
        entries.push(AdviceBuildEntry {
            tag: stem.to_string(),
            rel_path: format!("{ADVICE_DIR}/{stem}.md"),
            description,
        });
    }
    assert!(
        !entries.is_empty(),
        "no .md files under {}",
        advice_dir.display()
    );
    entries
}

pub fn is_advice_markdown(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    if name.starts_with('.') {
        return false;
    }
    path.extension().and_then(|e| e.to_str()) == Some("md")
}

fn validate_tag(tag: &str) {
    let mut chars = tag.chars();
    let Some(first) = chars.next() else {
        panic!("advice tag must not be empty");
    };
    assert!(
        first.is_ascii_lowercase(),
        "advice tag `{tag}` must start with a lowercase letter"
    );
    assert!(
        tag.len() <= MAX_TAG_LEN,
        "advice tag `{tag}` exceeds {MAX_TAG_LEN} characters"
    );
    assert!(
        chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
        "advice tag `{tag}` must be [a-z][a-z0-9_]* (lowercase letters, digits, underscore)"
    );
}

fn assert_no_duplicate_tags(entries: &[AdviceBuildEntry]) {
    let mut seen = std::collections::HashSet::new();
    for e in entries {
        assert!(
            seen.insert(e.tag.as_str()),
            "duplicate advice tag `{}`",
            e.tag
        );
    }
}

fn description_for(content: &str, tag: &str) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let Some(value) = first_line.strip_prefix("description:") else {
        panic!("advice `{tag}`: first line must be `description: ...`, got: {first_line:?}");
    };
    let desc = strip_optional_quotes(value.trim());
    assert!(
        !desc.is_empty(),
        "advice `{tag}`: description after `description:` is empty"
    );
    assert!(
        word_count(desc) <= MAX_DESC_WORDS,
        "advice `{tag}`: description has more than {MAX_DESC_WORDS} words: {desc}"
    );
    append_file_stats(desc, content)
}

const fn count_noun<'a>(n: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if n == 1 { singular } else { plural }
}

fn append_file_stats(desc: &str, content: &str) -> String {
    let lines = content.lines().count();
    let words = word_count(content);
    let line_word = count_noun(lines, "line", "lines");
    format!("{desc} [{lines} {line_word}, {words} words]")
}

fn strip_optional_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

fn word_count(s: &str) -> usize {
    s.split_whitespace().count()
}

fn render_registry(entries: &[AdviceBuildEntry]) -> String {
    let mut out = String::from("static ADVICE_ENTRIES: &[AdviceEntry] = &[\n");
    for e in entries {
        out.push_str("    AdviceEntry {\n");
        let _ = writeln!(out, "        tag: \"{}\",", escape_rust_str(&e.tag));
        let _ = writeln!(
            out,
            "        body: include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")),",
            escape_rust_str(&e.rel_path)
        );
        let _ = writeln!(
            out,
            "        description: \"{}\",",
            escape_rust_str(&e.description)
        );
        out.push_str("    },\n");
    }
    out.push_str("];\n");
    out
}

fn escape_rust_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{{{:x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{append_file_stats, is_advice_markdown};
    use std::path::Path;

    #[test]
    fn file_stats_use_line_when_count_is_one() {
        let stats = append_file_stats("One", "only one\n");
        assert_eq!(stats, "One [1 line, 2 words]");
    }

    #[test]
    fn file_stats_use_lines_when_count_is_not_one() {
        let many = append_file_stats("Many", "a\nb\n");
        assert_eq!(many, "Many [2 lines, 2 words]");
        let none = append_file_stats("None", "");
        assert_eq!(none, "None [0 lines, 0 words]");
    }

    #[test]
    fn editor_lock_files_are_not_advice_sources() {
        assert!(is_advice_markdown(Path::new(
            "assets/prompts/advice/fig_design.md"
        )));
        assert!(!is_advice_markdown(Path::new(
            "assets/prompts/advice/.#fig_design.md"
        )));
        assert!(!is_advice_markdown(Path::new(
            "assets/prompts/advice/fig_design.md~"
        )));
    }
}
