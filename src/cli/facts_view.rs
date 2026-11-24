use std::io::{self, Write};
use std::path::Path;

use super::fact_record::{FactRecord, facts_path, parse_facts};
use super::tag_list::{limit_words, list_line, write_body, write_list};

const NO_FACTS_MESSAGE: &str = "No facts saved yet.";

impl FactRecord {
    fn short_description(&self) -> String {
        limit_words(self.description.as_deref().unwrap_or(&self.fact))
    }

    fn body(&self) -> String {
        let mut text = format!("{}\n\n{}\n", self.short_description(), self.fact);
        if !self.symptoms.is_empty() {
            text.push_str("\nSymptoms:\n");
            for symptom in &self.symptoms {
                text.push_str("- ");
                text.push_str(symptom);
                text.push('\n');
            }
        }
        text
    }
}

fn read_facts_at(path: &Path) -> Result<Vec<FactRecord>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    Ok(parse_facts(&text).collect())
}

fn write_facts_list_at(path: &Path, out: impl Write) -> Result<(), String> {
    let facts = read_facts_at(path)?;
    let mut lines: Vec<String> = facts
        .iter()
        .map(|f| list_line(&f.fact_id, &f.short_description()))
        .collect();
    if lines.is_empty() {
        lines.push(NO_FACTS_MESSAGE.to_string());
    }
    write_list(out, "--facts", &lines)
}

fn write_fact_at(path: &Path, tag: &str, out: impl Write) -> Result<(), String> {
    let facts = read_facts_at(path)?;
    let Some(fact) = facts.iter().find(|f| f.fact_id == tag) else {
        let known: Vec<&str> = facts.iter().map(|f| f.fact_id.as_str()).collect();
        let known = if known.is_empty() {
            NO_FACTS_MESSAGE.trim_end_matches('.').to_lowercase()
        } else {
            known.join(", ")
        };
        return Err(format!("unknown --facts TAG `{tag}` (known: {known})"));
    };
    write_body(out, &fact.body())
}

pub(crate) fn print_facts_list_to_writer(out: impl Write) -> Result<(), String> {
    write_facts_list_at(&facts_path(), out)
}

pub(crate) fn print_fact_to_writer(tag: &str, out: impl Write) -> Result<(), String> {
    write_fact_at(&facts_path(), tag, out)
}

pub(crate) fn print_facts_list() -> Result<(), String> {
    print_facts_list_to_writer(io::stdout().lock())
}

pub(crate) fn print_fact(tag: &str) -> Result<(), String> {
    print_fact_to_writer(tag, io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_facts(content: &[u8]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("facts.jsonl");
        std::fs::write(&path, content).expect("write facts");
        (dir, path)
    }

    fn render(path: &Path, tag: Option<&str>) -> Result<String, String> {
        let mut buf = Vec::new();
        match tag {
            Some(t) => write_fact_at(path, t, &mut buf)?,
            None => write_facts_list_at(path, &mut buf)?,
        }
        Ok(String::from_utf8(buf).expect("utf8"))
    }

    #[test]
    fn facts_list_and_body_follow_advice_format_and_tolerate_missing_or_empty_files() {
        let (_dir, path) = temp_facts(
            b"{\"fact_id\":\"a\",\"fact\":\"one two\",\"description\":\"w1 w2 w3 w4 w5 w6 w7 w8 w9\",\"symptoms\":[\"s\"]}\n\nnot json\n{\"fact_id\":\"b\",\"fact\":\"x1 x2 x3 x4 x5 x6 x7 x8\"}",
        );
        assert_eq!(
            render(&path, None).expect("list"),
            "Type `malvin --facts TAG` to see a full document.\n\nTAG: Description\na: w1 w2 w3 w4 w5 w6 w7\nb: x1 x2 x3 x4 x5 x6 x7\n\n"
        );
        let body = render(&path, Some("a")).expect("body");
        assert_eq!(body, "w1 w2 w3 w4 w5 w6 w7\n\none two\n\nSymptoms:\n- s\n");
        let err = render(&path, Some("zzz")).expect_err("unknown");
        assert!(err.contains("known: a, b"), "{err}");
        let missing = path.with_file_name("absent.jsonl");
        for target in [Some("a"), None] {
            let out = render(&missing, target);
            match target {
                None => assert!(out.expect("list").contains(NO_FACTS_MESSAGE)),
                Some(_) => assert!(out.expect_err("unknown").contains("no facts saved yet")),
            }
        }
        for content in [&b""[..], &b"\n  \n"[..]] {
            let (_d, p) = temp_facts(content);
            assert!(render(&p, None).expect("empty").contains(NO_FACTS_MESSAGE));
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let err = render(dir.path(), None).expect_err("directory");
        assert!(err.contains("cannot read"), "{err}");
        record_round_trips_and_skips_malformed_lines();
    }

    fn record_round_trips_and_skips_malformed_lines() {
        use super::super::fact_record::FIELDS;
        let fact = FactRecord {
            fact_id: "a".into(),
            description: Some("d".into()),
            fact: "f".into(),
            symptoms: vec!["s".into()],
        };
        let line = serde_json::to_string(&fact).unwrap();
        let text = format!("{line}\nnot json\n{{\"fact_id\":\"b\",\"fact\":7}}\n{{\"fact_id\":\"c\",\"fact\":\"x\"}}\n");
        let ids: Vec<String> = parse_facts(&text).map(|f| f.fact_id).collect();
        assert_eq!(ids, ["a", "c"]);
        assert_eq!(FactRecord::from_line(&line), Some(fact));
        for field in ["fact_id", "description", "fact", "symptoms"] {
            assert!(line.contains(&format!("\"{field}\"")));
            assert!(FIELDS.contains(field));
        }
    }
}
