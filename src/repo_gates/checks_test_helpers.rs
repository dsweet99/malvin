//! Shared git/checks helpers for integration tests (unique module path for kiss disambiguation).

#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub(crate) fn git_init(work: &Path) {
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(work)
            .status()
            .expect("git init status")
            .success(),
        "git init failed in {}",
        work.display()
    );
}

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
    let path = work.join(crate::repo_gates::MALVIN_CHECKS_FILE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir legacy checks parent");
    }
    std::fs::write(path, content.as_ref()).expect("write legacy checks");
}
