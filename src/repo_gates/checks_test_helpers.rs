#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub(crate) use crate::test_support::test_utils::git_init;

#[cfg(test)]
pub(crate) fn write_git_root_checks(work: &Path, content: impl AsRef<[u8]>) {
    git_init(work);
    let path = crate::malvin_checks_path(work);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir checks parent");
    }
    std::fs::write(path, content.as_ref()).expect("write checks");
}

#[cfg(test)]
pub(crate) fn write_legacy_cwd_checks(work: &Path, content: impl AsRef<[u8]>) {
    let path = work.join(crate::workspace::workspace_paths::MALVIN_CHECKS_LEGACY_REL);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir legacy checks parent");
    }
    std::fs::write(path, content.as_ref()).expect("write legacy checks");
}
