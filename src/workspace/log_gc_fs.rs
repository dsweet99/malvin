use std::path::{Path, PathBuf};

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::output::{print_log_error, print_log_warning};

pub fn run_dir_timestamp(name: &str) -> Option<DateTime<Utc>> {
    let stamp = name.get(..15)?;
    let naive = NaiveDateTime::parse_from_str(stamp, "%Y%m%d_%H%M%S").ok()?;
    Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

pub(crate) fn is_run_log_dir_name(name: &str) -> bool {
    const STAMP_LEN: usize = 15;
    const TOKEN_LEN: usize = 8;
    if name.len() != STAMP_LEN + 1 + TOKEN_LEN {
        return false;
    }
    if name.as_bytes()[STAMP_LEN] != b'_' {
        return false;
    }
    if run_dir_timestamp(name).is_none() {
        return false;
    }
    name[STAMP_LEN + 1..]
        .bytes()
        .all(|b| b.is_ascii_alphanumeric())
}

pub fn dir_size(path: &Path) -> u64 {
    dir_size_inner(path).unwrap_or(0)
}

pub(crate) fn dir_size_inner(path: &Path) -> std::io::Result<u64> {
    let meta = std::fs::symlink_metadata(path)?;
    let ft = meta.file_type();
    if ft.is_symlink() {
        return Ok(meta.len());
    }
    if !ft.is_dir() {
        return Ok(if ft.is_file() { meta.len() } else { 0 });
    }
    let mut total = 0u64;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        total = total.saturating_add(dir_size_inner(&entry.path())?);
    }
    Ok(total)
}

pub(crate) fn remove_tree(path: &Path) -> Result<(), (PathBuf, std::io::Error)> {
    let fail = |e: std::io::Error| (path.to_path_buf(), e);
    if std::fs::symlink_metadata(path).map_err(fail)?.is_dir() {
        for entry in std::fs::read_dir(path).map_err(fail)? {
            remove_tree(&entry.map_err(fail)?.path())?;
        }
        std::fs::remove_dir(path).map_err(fail)
    } else {
        std::fs::remove_file(path).map_err(fail)
    }
}

pub(crate) fn report_undeletable(path: &Path, e: &std::io::Error) {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    print_log_error(&format!(
        "log GC skipped the rest of this log folder: could not delete {}: {e}",
        abs.display()
    ));
}

pub(crate) fn remove_tree_or_report(path: &Path) -> bool {
    match remove_tree(path) {
        Ok(()) => true,
        Err((stuck, e)) => {
            report_undeletable(&stuck, &e);
            false
        }
    }
}

pub(crate) fn list_run_dirs(logs_root: &Path) -> Vec<PathBuf> {
    let mut runs = Vec::new();
    let entries = match std::fs::read_dir(logs_root) {
        Ok(e) => e,
        Err(e) => {
            print_log_warning(&format!("could not list {}: {e}", logs_root.display()));
            return runs;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let is_run = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(is_run_log_dir_name);
        if path.is_dir() && is_run {
            runs.push(path);
        }
    }
    runs
}
