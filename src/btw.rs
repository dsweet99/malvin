
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const START: &str = "__MALVIN_BTW_START__";
const END: &str = "__MALVIN_BTW_END__";
const BTW_SUBDIR: &str = "btw";
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn dir(run_dir: &Path) -> PathBuf {
    run_dir.join(crate::malvin_constants::MAIN_DIR).join(BTW_SUBDIR)
}

#[must_use]
pub(crate) fn extract_fence(output: &str) -> Option<String> {
    let mut lines = output.lines();
    lines.find(|line| line.trim() == START)?;
    let mut body = Vec::new();
    for line in lines {
        if line.trim() == END {
            let text = body.join("\n");
            let text = text.trim();
            return (!text.is_empty()).then(|| text.to_string());
        }
        if line.trim() == START {
            body.clear();
            continue;
        }
        body.push(line);
    }
    None
}

pub(crate) fn send(run_dir: &Path, text: &str) -> Result<PathBuf, String> {
    let dir = dir(run_dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = dir.join(format!("{nanos:020}_{}_{n}.txt", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{text}").map_err(|e| e.to_string())?;
    Ok(path)
}

pub(crate) struct Pending {
    paths: Vec<PathBuf>,
    pub(crate) prompt: String,
}

impl Pending {
    pub(crate) fn mark_delivered(&self) {
        for path in &self.paths {
            let _ = std::fs::rename(path, path.with_extension("delivered"));
        }
    }
}

fn pending_paths(run_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = run_dir
        .and_then(|d| std::fs::read_dir(dir(d)).ok())
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    paths.sort();
    paths
}

#[must_use]
pub(crate) fn with_pending(run_dir: Option<&Path>, prompt: &str) -> Pending {
    let mut out = String::new();
    let mut taken = Vec::new();
    for path in pending_paths(run_dir) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        out.extend(["[btw] ", text.trim(), "\n\n"]);
        taken.push(path);
    }
    out.push_str(prompt);
    Pending {
        paths: taken,
        prompt: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_requires_markers_alone_on_lines() {
        let ok = format!("hi\n{START}\nuse x\nmore\n{END}\nbye");
        assert_eq!(extract_fence(&ok).as_deref(), Some("use x\nmore"));
        assert_eq!(extract_fence(&format!("a {START}\nx\n{END}")), None);
        assert_eq!(extract_fence(&format!("{START}\n{END}")), None);
        assert_eq!(extract_fence(&format!("{START}\nx")), None);
        assert_eq!(extract_fence("nothing"), None);
    }

    #[test]
    fn notes_are_delivered_once_and_kept() {
        let tmp = tempfile::tempdir().unwrap();
        let a = send(tmp.path(), "one").unwrap();
        let b = send(tmp.path(), "two").unwrap();
        assert_ne!(a, b);
        let first = with_pending(Some(tmp.path()), "PROMPT");
        assert_eq!(first.prompt, "[btw] one\n\n[btw] two\n\nPROMPT");
        let again = with_pending(Some(tmp.path()), "PROMPT");
        assert_eq!(again.prompt, first.prompt);
        again.mark_delivered();
        assert_eq!(with_pending(Some(tmp.path()), "P").prompt, "P");
        assert!(a.with_extension("delivered").is_file());
        assert_eq!(with_pending(None, "P").prompt, "P");
    }
}
