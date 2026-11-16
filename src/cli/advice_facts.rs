use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

pub(crate) const FACTS_TAG: &str = "facts";
const FACTS_DESCRIPTION: &str = "Time-saving facts you've discovered";
const FACTS_FILE: &str = "facts.jsonl";
const NO_FACTS_MESSAGE: &str = "No facts saved yet.";

pub(crate) fn facts_path() -> PathBuf {
    crate::malvin_user_home_root().join(FACTS_FILE)
}

fn for_each_chunk(
    mut reader: impl Read,
    mut f: impl FnMut(&[u8]) -> io::Result<()>,
) -> io::Result<u8> {
    let mut buf = vec![0u8; 64 * 1024];
    let mut last = b'\n';
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => return Ok(last),
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        f(&buf[..n])?;
        last = buf[n - 1];
    }
}

fn count_lines_in_reader(reader: impl Read) -> io::Result<u64> {
    let mut lines = 0u64;
    let last = for_each_chunk(reader, |chunk| {
        lines += chunk.iter().map(|&b| u64::from(b == b'\n')).sum::<u64>();
        Ok(())
    })?;
    Ok(lines + u64::from(last != b'\n'))
}

fn count_lines_at(path: &Path) -> u64 {
    File::open(path)
        .and_then(count_lines_in_reader)
        .unwrap_or(0)
}

fn facts_list_line_at(path: &Path) -> String {
    let n = count_lines_at(path);
    format!("{FACTS_TAG}: {FACTS_DESCRIPTION} [{n} facts]")
}

pub(crate) fn facts_list_line() -> String {
    facts_list_line_at(&facts_path())
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_fact_line(line: &str) -> String {
    let parsed: Option<serde_json::Value> = serde_json::from_str(line).ok();
    let field = |key: &str| {
        parsed
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(serde_json::Value::as_str)
    };
    match (field("fact_id"), field("fact")) {
        (Some(id), Some(fact)) => format!("{id}: {}", one_line(fact)),
        (None, Some(fact)) => one_line(fact),
        _ => one_line(line),
    }
}

fn write_facts_at(path: &Path, mut out: impl Write) -> Result<(), String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    let mut any = false;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        any = true;
        writeln!(out, "{}", render_fact_line(line)).map_err(|e| format!("stdout: {e}"))?;
    }
    if !any {
        writeln!(out, "{NO_FACTS_MESSAGE}").map_err(|e| format!("stdout: {e}"))?;
    }
    Ok(())
}

pub(crate) fn write_facts(out: impl Write) -> Result<(), String> {
    write_facts_at(&facts_path(), out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_facts(content: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join(FACTS_FILE);
        std::fs::write(&path, content).expect("write facts");
        (dir, path)
    }

    #[test]
    fn counts_lines_with_and_without_trailing_newline() {
        assert_eq!(count_lines_in_reader(&b""[..]).expect("empty"), 0);
        assert_eq!(count_lines_in_reader(&b"a\nb\n"[..]).expect("trailing"), 2);
        assert_eq!(count_lines_in_reader(&b"a\nb"[..]).expect("no trailing"), 2);
    }

    #[test]
    fn list_line_reports_count_and_zero_when_missing() {
        let (_dir, path) = temp_facts(b"{\"id\":1}\n{\"id\":2}\n{\"id\":3}\n");
        assert_eq!(
            facts_list_line_at(&path),
            "facts: Time-saving facts you've discovered [3 facts]"
        );
        let missing = path.with_file_name("absent.jsonl");
        assert_eq!(
            facts_list_line_at(&missing),
            "facts: Time-saving facts you've discovered [0 facts]"
        );
    }

    #[test]
    fn write_facts_renders_one_line_per_fact_and_reports_none_when_missing_or_empty() {
        let (_dir, path) = temp_facts(
            b"{\"fact_id\":\"a\",\"fact\":\"one\\ntwo  three\",\"symptoms\":[\"s\"]}\n\nnot json\n{\"fact_id\":\"b\",\"fact\":\"four\"}",
        );
        let mut buf = Vec::new();
        write_facts_at(&path, &mut buf).expect("render");
        assert_eq!(
            String::from_utf8(buf).expect("utf8"),
            "a: one two three\nnot json\nb: four\n"
        );
        let missing = path.with_file_name("absent.jsonl");
        let mut buf = Vec::new();
        write_facts_at(&missing, &mut buf).expect("missing is ok");
        assert_eq!(buf, b"No facts saved yet.\n");
        for content in [&b""[..], &b"\n  \n"[..]] {
            let (_d, p) = temp_facts(content);
            let mut buf = Vec::new();
            write_facts_at(&p, &mut buf).expect("empty is ok");
            assert_eq!(buf, b"No facts saved yet.\n");
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let err = write_facts_at(dir.path(), Vec::new()).expect_err("directory");
        assert!(err.contains("cannot read"), "{err}");
    }
}
