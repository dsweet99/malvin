//! Restore canonical kiss/git ignore rules after session dotfile restore.

use std::path::Path;

const TPL_KISSIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/kissignore"
));
const TPL_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));

const OPS_IGNORE_LINE: &str = "ops/";

fn ignore_lines(content: &str) -> impl Iterator<Item = &str> {
    content.lines().map(str::trim).filter(|line| !line.is_empty())
}

fn covers_ops_prefix(content: &str) -> bool {
    ignore_lines(content).any(|line| line == OPS_IGNORE_LINE)
}

fn reconcile_file_if_missing_ops(
    path: &Path,
    template: &str,
    write_err: &str,
) -> Result<(), String> {
    let current = if path.is_file() {
        std::fs::read_to_string(path).map_err(|e| format!("read {}: {e}", path.display()))?
    } else {
        String::new()
    };
    if covers_ops_prefix(&current) {
        return Ok(());
    }
    std::fs::write(path, template).map_err(|e| format!("{write_err}: {e}"))
}

/// After session dotfile restore, re-apply canonical ignore templates when `ops/` is missing.
///
/// Gate loops snapshot a drifted root `.kissignore` (often `target/` only). Restore-before-gates
/// then re-exposes `ops/` Modal scripts to strict root `.kissconfig`. Templates in
/// `default_repo/` exclude `ops/`; kiss also honors root `.gitignore`.
pub fn reconcile_workspace_ignore_files(work_dir: &Path) -> Result<(), String> {
    if !template_covers_ops() {
        return Ok(());
    }
    reconcile_file_if_missing_ops(
        &work_dir.join(super::KISSIGNORE_FILE),
        TPL_KISSIGNORE,
        "reconcile .kissignore",
    )?;
    reconcile_file_if_missing_ops(
        &work_dir.join(".gitignore"),
        TPL_GITIGNORE,
        "reconcile .gitignore",
    )
}

fn template_covers_ops() -> bool {
    covers_ops_prefix(TPL_KISSIGNORE) || covers_ops_prefix(TPL_GITIGNORE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_exclude_ops_from_kiss() {
        assert!(covers_ops_prefix(TPL_KISSIGNORE));
    }

    #[test]
    fn reconcile_restores_drifted_kissignore_and_gitignore() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        std::fs::write(work.join(".kissignore"), "target/\n").unwrap();
        std::fs::write(work.join(".gitignore"), "target\n").unwrap();

        reconcile_workspace_ignore_files(work).unwrap();

        assert_eq!(
            std::fs::read_to_string(work.join(".kissignore")).unwrap(),
            TPL_KISSIGNORE
        );
        assert_eq!(
            std::fs::read_to_string(work.join(".gitignore")).unwrap(),
            TPL_GITIGNORE
        );
    }

    #[test]
    fn reconcile_skips_when_ops_already_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path();
        let kiss = "target/\nops/\n";
        let git = "target/\nops/\n";
        std::fs::write(work.join(".kissignore"), kiss).unwrap();
        std::fs::write(work.join(".gitignore"), git).unwrap();

        reconcile_workspace_ignore_files(work).unwrap();

        assert_eq!(std::fs::read_to_string(work.join(".kissignore")).unwrap(), kiss);
        assert_eq!(std::fs::read_to_string(work.join(".gitignore")).unwrap(), git);
    }
}
