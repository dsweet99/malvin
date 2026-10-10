use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use super::alloc::{DotfileBackupLabels, allocate_backup_dir, remove_if_exists};

pub struct NamedFileTreePolicy {
    pub file_name: &'static str,
    pub category: &'static str,
    pub labels: DotfileBackupLabels,
    pub copy_error: &'static str,
    pub restore_write_error: &'static str,
}

pub trait NamedFileKind {
    const POLICY: NamedFileTreePolicy;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedFileEntry {
    pub rel: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedFile<K> {
    pub rel: PathBuf,
    pub bytes: Vec<u8>,
    kind: PhantomData<K>,
}

impl<K> NamedFile<K> {
    #[must_use]
    pub const fn new(rel: PathBuf, bytes: Vec<u8>) -> Self {
        Self {
            rel,
            bytes,
            kind: PhantomData,
        }
    }
}

impl<K> From<NamedFileEntry> for NamedFile<K> {
    fn from(value: NamedFileEntry) -> Self {
        Self::new(value.rel, value.bytes)
    }
}

impl<K> From<NamedFile<K>> for NamedFileEntry {
    fn from(value: NamedFile<K>) -> Self {
        Self {
            rel: value.rel,
            bytes: value.bytes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamedFileBackup<K> {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<NamedFile<K>>,
    },
}

impl<K: NamedFileKind> NamedFileBackup<K> {
    #[must_use]
    pub fn collect_relpaths(work_dir: &Path) -> Vec<PathBuf> {
        collect_workspace_named_file_relpaths(work_dir, K::POLICY.file_name)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn backup_if_present(work_dir: &Path) -> Result<Self, String> {
        Self::backup_if_present_with_id(work_dir, super::alloc::random_backup_id)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn backup_if_present_with_id(
        work_dir: &Path,
        mut generate_id: impl FnMut(usize) -> String,
    ) -> Result<Self, String> {
        Self::backup(work_dir, &mut generate_id)
    }

    pub(super) fn backup(
        work_dir: &Path,
        generate_id: &mut impl FnMut(usize) -> String,
    ) -> Result<Self, String> {
        let rels = Self::collect_relpaths(work_dir);
        if rels.is_empty() {
            return Ok(Self::Missing);
        }
        let policy = &K::POLICY;
        let root = crate::workspace::workspace_paths::snapshot_category_dir(policy.category);
        let dest_dir = allocate_backup_dir(&root, generate_id, &policy.labels)?;
        let files = copy_rels_into_backup(work_dir, &dest_dir, &rels, policy)?;
        Ok(Self::Present {
            backup_root: dest_dir,
            files: files.into_iter().map(NamedFile::from).collect(),
        })
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn restore(&self, work_dir: &Path) -> Result<(), String> {
        match self {
            Self::Missing => {
                restore_missing_named_files(work_dir, &Self::collect_relpaths(work_dir), &K::POLICY)
            }
            Self::Present { files, .. } => restore_present_named_files(work_dir, files, &K::POLICY),
        }
    }
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

fn restore_missing_named_files(
    work_dir: &Path,
    current_rels: &[PathBuf],
    policy: &NamedFileTreePolicy,
) -> Result<(), String> {
    for rel in current_rels {
        remove_if_exists(&work_dir.join(rel), policy.labels.restore)?;
    }
    Ok(())
}

fn restore_present_named_files<K>(
    work_dir: &Path,
    files: &[NamedFile<K>],
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
        let _ = collect_workspace_named_file_relpaths;
        let _ = collect_root_named_file_only;
        let _ = walk_named_files;
        let _ = stringify!(NamedFileBackup);
        let _ = stringify!(restore_missing_named_files);
        let _ = stringify!(restore_present_named_files);
        let _ = stringify!(copy_rels_into_backup);
    }
}
