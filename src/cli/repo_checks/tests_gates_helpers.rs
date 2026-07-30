use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(super) fn write_executable_script(bin_dir: &Path, name: &str, body: &str) {
    let path = bin_dir.join(name);
    fs::write(&path, body).expect("write script");
    let mut perms = fs::metadata(&path).expect("script meta").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod script");
}

pub(super) fn write_trace_echo_script(bin_dir: &Path, name: &str, trace: &Path, exit_code: i32) {
    let body = format!(
        "#!/bin/sh\necho \"{name} $@\" >> \"{}\"\nexit {exit_code}\n",
        trace.display()
    );
    write_executable_script(bin_dir, name, &body);
}

pub(super) fn install_trace_echo_bins(
    bin_dir: &Path,
    trace: &Path,
    names: &[&str],
    exit_code: i32,
) {
    for name in names {
        write_trace_echo_script(bin_dir, name, trace, exit_code);
    }
}

pub(super) fn git_init_work(work: &Path) {
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

pub(super) fn workspace_git_minimal_cargo_rs_py_tests(work: &Path) {
    git_init_work(work);
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .expect("Cargo.toml");
    fs::write(work.join("main.rs"), "fn main() {}").expect("main.rs");
    fs::write(work.join("script.py"), "print('ok')").expect("script.py");
    fs::create_dir(work.join("tests")).expect("tests/");
}

pub(super) fn workspace_git_cargo_main_only(work: &Path) {
    git_init_work(work);
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .expect("Cargo.toml");
    fs::write(work.join("main.rs"), "fn main() {}").expect("main.rs");
}

pub(super) fn workspace_git_kissconfig_90_cargo_rs_py(work: &Path) {
    workspace_git_minimal_cargo_rs_py_tests(work);
    fs::write(
        work.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 90\n",
    )
    .expect(".kissconfig");
}

pub(super) fn workspace_git_malvin_checks_line(work: &Path, line: &str) {
    git_init_work(work);
    let path = crate::malvin_checks_path(work);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("checks parent");
    }
    fs::write(path, line).expect("checks");
}

pub(super) fn workspace_git_precommit_malvin_checks_cargo_main(work: &Path) {
    workspace_git_cargo_main_only(work);
    fs::write(work.join(".pre-commit-config.yaml"), "repos:\n").expect("pre-commit");
    let path = crate::malvin_checks_path(work);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("checks parent");
    }
    fs::write(path, "custom --only\n").expect("checks");
}
