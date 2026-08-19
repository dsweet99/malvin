#[cfg(test)]
pub fn install_exit_gate_bin(bin_dir: &std::path::Path, name: &str, code: i32) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = bin_dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\nexit {code}\n")).expect("write fake bin");
        let mut perms = std::fs::metadata(&path).expect("bin meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&path, perms).expect("chmod fake bin");
    }
    #[cfg(windows)]
    {
        let path = bin_dir.join(format!("{name}.cmd"));
        std::fs::write(&path, format!("@exit {code}\r\n")).expect("write fake bin");
    }
    #[cfg(not(any(unix, windows)))]
    {
        let path = bin_dir.join(name);
        std::fs::write(&path, format!("#!/bin/sh\nexit {code}\n")).expect("write fake bin");
    }
}

#[cfg(test)]
pub fn write_fake_gate(
    work_dir: &std::path::Path,
    gate_name: &str,
    exit_code: i32,
) -> (tempfile::TempDir, crate::repo_checks::FakeCommandDirGuard) {
    if crate::git_worktree_toplevel(work_dir).is_none() {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(work_dir)
            .status();
    }
    let checks = crate::malvin_checks_path(work_dir);
    if let Some(parent) = checks.parent() {
        std::fs::create_dir_all(parent).expect("mkdir checks parent");
    }
    std::fs::write(checks, format!("{gate_name}\n")).expect("checks");
    let bin_dir = tempfile::tempdir().expect("bindir");
    install_exit_gate_bin(bin_dir.path(), gate_name, exit_code);
    let guard = crate::repo_checks::set_fake_command_dir(bin_dir.path());
    (bin_dir, guard)
}

#[cfg(test)]
mod write_fake_gate_tests {
    use super::write_fake_gate;

    #[test]
    fn write_fake_gate_seeds_checks_on_workspace_without_malvin_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let work = tmp.path().join("fresh");
        std::fs::create_dir_all(&work).expect("mkdir work");
        let (_bin, _guard) = write_fake_gate(&work, "kiss", 0);
        assert!(crate::malvin_checks_path(&work).is_file());
    }
}
