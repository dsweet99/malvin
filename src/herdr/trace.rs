//! Best-effort herdr failure lines in the active run directory.

use std::io::Write;
use std::path::Path;

/// Append one failure line to `herdr.log` under `run_dir` (create if needed).
pub fn log_herdr_failure(run_dir: Option<&Path>, phase: &str, detail: &str) {
    let Some(dir) = run_dir else {
        return;
    };
    let path = dir.join("herdr.log");
    let line = format!("herdr {phase} failed: {detail}\n");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(line.as_bytes()));
}

#[cfg(test)]
mod tests {
    use super::log_herdr_failure;
    use tempfile::tempdir;

    #[test]
    fn log_herdr_failure_appends_line() {
        let dir = tempdir().expect("tempdir");
        log_herdr_failure(None, "bind", "ignored");
        log_herdr_failure(Some(dir.path()), "bind", "socket error");
        let text = std::fs::read_to_string(dir.path().join("herdr.log")).expect("read");
        assert!(text.contains("herdr bind failed: socket error"), "{text}");
    }
}
