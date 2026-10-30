use std::path::Path;

use super::text;

pub(super) async fn until_exit(mut child: tokio::process::Child, run_dir: &Path) {
    let _stdin = child.stdin.take();
    let mut seen = String::new();
    loop {
        publish(run_dir, &mut seen);
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => break,
            Ok(None) => tokio::time::sleep(std::time::Duration::from_millis(50)).await,
        }
    }
    publish(run_dir, &mut seen);
}

fn publish(run_dir: &Path, seen: &mut String) {
    let current = fingerprint(run_dir);
    if current.is_empty() || current == *seen {
        return;
    }
    *seen = current;
    crate::herdr::notify_summary_ready(run_dir);
}

fn fingerprint(run_dir: &Path) -> String {
    let task = std::fs::read_to_string(text::task_path(run_dir)).unwrap_or_default();
    let status = std::fs::read_to_string(text::status_path(run_dir)).unwrap_or_default();
    if task.is_empty() && status.is_empty() {
        String::new()
    } else {
        format!("{task}\n{status}")
    }
}
