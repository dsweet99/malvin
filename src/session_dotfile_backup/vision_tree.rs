use super::alloc::DotfileBackupLabels;
use super::named_file_tree::{NamedFile, NamedFileBackup, NamedFileKind, NamedFileTreePolicy};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisionKind;

impl NamedFileKind for VisionKind {
    const POLICY: NamedFileTreePolicy = NamedFileTreePolicy {
        file_name: "VISION.md",
        category: "vision",
        labels: DotfileBackupLabels {
            mkdir: "vision backup mkdir",
            collision: "vision backup mkdir",
            restore: "vision restore",
        },
        copy_error: "VISION.md backup copy",
        restore_write_error: "vision restore",
    };
}

pub type VisionBackup = NamedFileBackup<VisionKind>;
pub type VisionFileBackup = NamedFile<VisionKind>;

#[cfg(test)]
#[path = "vision_tree_tests.rs"]
pub(crate) mod vision_tree_tests;
