use super::alloc::DotfileBackupLabels;
use super::named_file_tree::{NamedFile, NamedFileBackup, NamedFileKind, NamedFileTreePolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitignoreKind;

impl NamedFileKind for GitignoreKind {
    const POLICY: NamedFileTreePolicy = NamedFileTreePolicy {
        file_name: ".gitignore",
        category: "gitignore",
        labels: DotfileBackupLabels {
            mkdir: "gitignore backup mkdir",
            collision: "gitignore backup mkdir",
            restore: "gitignore restore",
        },
        copy_error: ".gitignore backup copy",
        restore_write_error: "gitignore restore",
    };
}

pub type GitignoreBackup = NamedFileBackup<GitignoreKind>;
pub type GitignoreFileBackup = NamedFile<GitignoreKind>;

#[cfg(test)]
#[path = "gitignore_tree_tests.rs"]
pub(crate) mod gitignore_tree_tests;
