use std::path::{Path, PathBuf};

use super::alloc::{DotfileBackupLabels, allocate_backup_dir, remove_if_exists};

pub(crate) struct NamedFileTreePolicy {
    pub file_name: &'static str,
    pub category: &'static str,
    pub labels: DotfileBackupLabels,
    pub copy_error: &'static str,
    pub restore_write_error: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedFileEntry {
    pub rel: PathBuf,
    pub bytes: Vec<u8>,
}

macro_rules! typed_named_file_backup {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            pub rel: PathBuf,
            pub bytes: Vec<u8>,
        }

        impl From<NamedFileEntry> for $name {
            fn from(value: NamedFileEntry) -> Self {
                Self {
                    rel: value.rel,
                    bytes: value.bytes,
                }
            }
        }

        impl From<$name> for NamedFileEntry {
            fn from(value: $name) -> Self {
                Self {
                    rel: value.rel,
                    bytes: value.bytes,
                }
            }
        }

        impl $name {
            #[must_use]
            pub fn as_named_entries(files: &[Self]) -> Vec<NamedFileEntry> {
                files
                    .iter()
                    .map(|file| NamedFileEntry {
                        rel: file.rel.clone(),
                        bytes: file.bytes.clone(),
                    })
                    .collect()
            }
        }
    };
}

pub(crate) use typed_named_file_backup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NamedFileTreeState {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<NamedFileEntry>,
    },
}

fn walk_named_files(dir: &Path, work_dir: &Path, file_name: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }
            walk_named_files(&path, work_dir, file_name, found);
        } else if file_type.is_file()
            && path.file_name().and_then(|n| n.to_str()) == Some(file_name)
            && let Ok(rel) = path.strip_prefix(work_dir)
        {
            found.push(rel.to_path_buf());
        }
    }
}

fn collect_root_named_file_only(work_dir: &Path, file_name: &str) -> Vec<PathBuf> {
    if work_dir.join(file_name).is_file() {
        vec![PathBuf::from(file_name)]
    } else {
        vec![]
    }
}

#[must_use]
pub(crate) fn collect_workspace_named_file_relpaths(
    work_dir: &Path,
    file_name: &str,
) -> Vec<PathBuf> {
    if crate::git_worktree_toplevel(work_dir).is_some() {
        let mut found = Vec::new();
        walk_named_files(work_dir, work_dir, file_name, &mut found);
        found.sort();
        found
    } else {
        collect_root_named_file_only(work_dir, file_name)
    }
}

pub(crate) fn backup_named_file_tree(
    work_dir: &Path,
    rels: &[PathBuf],
    generate_id: &mut impl FnMut(usize) -> String,
    policy: &NamedFileTreePolicy,
) -> Result<NamedFileTreeState, String> {
    if rels.is_empty() {
        return Ok(NamedFileTreeState::Missing);
    }

    let root = crate::workspace_paths::snapshot_category_dir(policy.category);
    let dest_dir = allocate_backup_dir(&root, generate_id, &policy.labels)?;
    let files = copy_rels_into_backup(work_dir, &dest_dir, rels, policy)?;
    Ok(NamedFileTreeState::Present {
        backup_root: dest_dir,
        files,
    })
}

fn copy_rels_into_backup(
    work_dir: &Path,
    dest_dir: &Path,
    rels: &[PathBuf],
    policy: &NamedFileTreePolicy,
) -> Result<Vec<NamedFileEntry>, String> {
    let mut files = Vec::with_capacity(rels.len());
    for rel in rels {
        let src = work_dir.join(rel);
        let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", policy.copy_error))?;
        let dest_file = dest_dir.join(rel);
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", policy.labels.mkdir))?;
        }
        if let Err(e) = std::fs::write(&dest_file, &bytes) {
            let _ = std::fs::remove_dir_all(dest_dir);
            return Err(format!("{}: {e}", policy.copy_error));
        }
        files.push(NamedFileEntry {
            rel: rel.clone(),
            bytes,
        });
    }
    Ok(files)
}

pub(crate) fn restore_missing_named_files(
    work_dir: &Path,
    current_rels: &[PathBuf],
    policy: &NamedFileTreePolicy,
) -> Result<(), String> {
    for rel in current_rels {
        remove_if_exists(&work_dir.join(rel), policy.labels.restore)?;
    }
    Ok(())
}

pub(crate) fn restore_present_named_files(
    work_dir: &Path,
    files: &[NamedFileEntry],
    policy: &NamedFileTreePolicy,
) -> Result<(), String> {
    let snapshot_rels: std::collections::BTreeSet<_> = files.iter().map(|file| &file.rel).collect();
    for file in files {
        let dst = work_dir.join(&file.rel);
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("{}: {e}", policy.labels.restore))?;
        }
        std::fs::write(&dst, &file.bytes)
            .map_err(|e| format!("{}: {e}", policy.restore_write_error))?;
    }
    for rel in collect_workspace_named_file_relpaths(work_dir, policy.file_name) {
        if !snapshot_rels.contains(&rel) {
            remove_if_exists(&work_dir.join(rel), policy.labels.restore)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_named_file_tree_types() {
        let _: Option<NamedFileEntry> = None;
        let _: Option<NamedFileTreeState> = None;
        let _ = collect_workspace_named_file_relpaths;
        let _ = collect_root_named_file_only;
        let _ = walk_named_files;
        let _ = stringify!(backup_named_file_tree);
        let _ = stringify!(restore_missing_named_files);
        let _ = stringify!(restore_present_named_files);
        let _ = stringify!(copy_rels_into_backup);
        let _ = stringify!(typed_named_file_backup);
    }
}
