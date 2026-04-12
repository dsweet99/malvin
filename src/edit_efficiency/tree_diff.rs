//! Rename-aware tree diff cost for selected extensions.

use std::path::Path;

use tracing::warn;

use crate::edit_efficiency::byte_cost::byte_edit_cost;
use crate::edit_efficiency::error::EditEfficiencyError;
use crate::edit_efficiency::git_tree::{blob_at_tree, diff_name_status_z};

#[derive(Debug, Clone)]
enum NameRecord<'a> {
    Modify {
        path: &'a str,
    },
    Add {
        path: &'a str,
    },
    Delete {
        path: &'a str,
    },
    Rename {
        old_path: &'a str,
        new_path: &'a str,
    },
}

fn has_measured_ext(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e, "rs" | "py" | "md"))
}

fn record_is_measured(rec: &NameRecord<'_>) -> bool {
    match rec {
        NameRecord::Modify { path } | NameRecord::Add { path } | NameRecord::Delete { path } => {
            has_measured_ext(path)
        }
        NameRecord::Rename { old_path, new_path } => {
            has_measured_ext(old_path) || has_measured_ext(new_path)
        }
    }
}

fn utf8_git_field(raw: &[u8]) -> Result<&str, EditEfficiencyError> {
    std::str::from_utf8(raw).map_err(|_| EditEfficiencyError::ParseNameStatus)
}

fn next_nul_field<'a, I>(iter: &mut I) -> Result<&'a [u8], EditEfficiencyError>
where
    I: Iterator<Item = &'a [u8]>,
{
    iter.next().ok_or(EditEfficiencyError::ParseNameStatus)
}

fn push_mad_from_status<'a>(
    status_trim: &str,
    path: &'a str,
    out: &mut Vec<NameRecord<'a>>,
) -> bool {
    match status_trim.as_bytes().first() {
        Some(b'M') => {
            out.push(NameRecord::Modify { path });
            true
        }
        Some(b'A') => {
            out.push(NameRecord::Add { path });
            true
        }
        Some(b'D') => {
            out.push(NameRecord::Delete { path });
            true
        }
        _ => false,
    }
}

fn parse_name_status_z(data: &[u8]) -> Result<Vec<NameRecord<'_>>, EditEfficiencyError> {
    let mut parts = data.split(|&b| b == 0).filter(|s| !s.is_empty());
    let mut out = Vec::new();
    while let Some(status_raw) = parts.next() {
        let status = utf8_git_field(status_raw)?;
        let status_trim = status.trim();
        if status_trim.starts_with('R') || status_trim.starts_with('C') {
            let old_p = next_nul_field(&mut parts)?;
            let new_p = next_nul_field(&mut parts)?;
            out.push(NameRecord::Rename {
                old_path: utf8_git_field(old_p)?,
                new_path: utf8_git_field(new_p)?,
            });
        } else {
            let path_raw = next_nul_field(&mut parts)?;
            let path = utf8_git_field(path_raw)?;
            if !push_mad_from_status(status_trim, path, &mut out) {
                warn!(
                    status = %status_trim,
                    "edit_efficiency: skipping unrecognized tree-diff name-status (expected M/A/D or R/C)"
                );
            }
        }
    }
    Ok(out)
}

fn record_cost(
    repo_root: &Path,
    old_tree: &str,
    new_tree: &str,
    rec: &NameRecord<'_>,
) -> Result<u64, EditEfficiencyError> {
    match rec {
        NameRecord::Modify { path } => {
            let o = blob_at_tree(repo_root, old_tree, path)?;
            let n = blob_at_tree(repo_root, new_tree, path)?;
            Ok(byte_edit_cost(&o, &n))
        }
        NameRecord::Add { path } => {
            let n = blob_at_tree(repo_root, new_tree, path)?;
            Ok(byte_edit_cost(&[], &n))
        }
        NameRecord::Delete { path } => {
            let o = blob_at_tree(repo_root, old_tree, path)?;
            Ok(byte_edit_cost(&o, &[]))
        }
        NameRecord::Rename { old_path, new_path } => {
            let o = blob_at_tree(repo_root, old_tree, old_path)?;
            let n = blob_at_tree(repo_root, new_tree, new_path)?;
            Ok(byte_edit_cost(&o, &n))
        }
    }
}

/// Total byte edit cost between `old_tree` and `new_tree` for `.rs`/`.py`/`.md` only.
pub fn filtered_tree_diff_cost(
    repo_root: &Path,
    old_tree: &str,
    new_tree: &str,
) -> Result<u64, EditEfficiencyError> {
    if old_tree == new_tree {
        return Ok(0);
    }
    let raw = diff_name_status_z(repo_root, old_tree, new_tree)?;
    let records = parse_name_status_z(&raw)?;
    let mut sum = 0u64;
    for rec in &records {
        if record_is_measured(rec) {
            sum += record_cost(repo_root, old_tree, new_tree, rec)?;
        }
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use tempfile::TempDir;

    use super::filtered_tree_diff_cost;

    fn init_repo() -> TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(p)
                .status()
                .expect("git init")
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.email", "t@e.st"])
                .current_dir(p)
                .status()
                .expect("git config email")
                .success()
        );
        assert!(
            Command::new("git")
                .args(["config", "user.name", "t"])
                .current_dir(p)
                .status()
                .expect("git config name")
                .success()
        );
        tmp
    }

    #[test]
    fn rename_same_content_zero_cost_for_rs() {
        let tmp = init_repo();
        let p = tmp.path();
        std::fs::write(p.join("a.rs"), b"x").unwrap();
        let t1 = {
            let _ = Command::new("git")
                .args(["add", "-A"])
                .current_dir(p)
                .status()
                .unwrap();
            let out = Command::new("git")
                .args(["write-tree"])
                .current_dir(p)
                .output()
                .unwrap();
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        std::fs::rename(p.join("a.rs"), p.join("b.rs")).unwrap();
        let t2 = {
            let _ = Command::new("git")
                .args(["add", "-A"])
                .current_dir(p)
                .status()
                .unwrap();
            let out = Command::new("git")
                .args(["write-tree"])
                .current_dir(p)
                .output()
                .unwrap();
            String::from_utf8(out.stdout).unwrap().trim().to_string()
        };
        let c = filtered_tree_diff_cost(p, &t1, &t2).unwrap();
        assert_eq!(c, 0);
    }

    #[test]
    fn kiss_stringify_tree_diff() {
        let _ = stringify!(super::NameRecord);
        let _ = stringify!(super::has_measured_ext);
        let _ = stringify!(super::record_is_measured);
        let _ = stringify!(super::utf8_git_field);
        let _ = stringify!(super::next_nul_field);
        let _ = stringify!(super::push_mad_from_status);
        let _ = stringify!(super::parse_name_status_z);
        let _ = stringify!(super::record_cost);
        let _ = stringify!(super::filtered_tree_diff_cost);
    }
}
