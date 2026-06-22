use std::fs;
use std::path::Path;

pub(super) fn workspace_git_minimal_cargo_rs_py_tests(work: &Path) {
    fs::create_dir(work.join(".git")).expect(".git");
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
    fs::create_dir(work.join(".git")).expect(".git");
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
    fs::create_dir(work.join(".git")).expect(".git");
    fs::create_dir_all(work.join(".malvin")).expect(".malvin");
    fs::write(work.join(".malvin/checks"), line).expect(".malvin/checks");
}

pub(super) fn workspace_git_precommit_malvin_checks_cargo_main(work: &Path) {
    workspace_git_cargo_main_only(work);
    fs::write(work.join(".pre-commit-config.yaml"), "repos:\n").expect("pre-commit");
    fs::create_dir_all(work.join(".malvin")).expect(".malvin");
    fs::write(work.join(".malvin/checks"), "custom --only\n").expect(".malvin/checks");
}
