mod common;

use std::path::Path;
use std::process::Command;

const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
));

const DEFAULT_PROMPTS_REVIEW_PLAN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_prompts/review_plan.md"
));

#[cfg(unix)]
fn run_root_help_output() -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.arg("--help");
    common::command_output_with_timeout(&mut cmd, common::MALVIN_TEST_CMD_TIMEOUT)
        .expect("malvin --help")
}

#[cfg(not(unix))]
fn run_root_help_output() -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .arg("--help")
        .output()
        .expect("malvin --help")
}

#[cfg(unix)]
fn contains_help_subcommand(help: &str, subcommand: &str) -> bool {
    help.lines()
        .any(|line| line.split_whitespace().next() == Some(subcommand))
}

#[cfg(unix)]
fn help_option_count(help: &str, option: &str) -> usize {
    help.lines()
        .filter(|line| line.split_whitespace().any(|token| token == option))
        .count()
}

#[cfg_attr(unix, test)]
fn help_lists_global_no_markdown_once() {
    let out = run_root_help_output();
    assert!(
        out.status.success(),
        "help failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let no_markdown_option_lines = help_option_count(&s, "--no-markdown");
    assert_eq!(
        no_markdown_option_lines, 1,
        "expected exactly one --no-markdown in root help: {s}"
    );
}

#[cfg_attr(unix, test)]
fn help_omits_removed_ground_and_sync_commands() {
    let out = run_root_help_output();
    assert!(
        out.status.success(),
        "help failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        !contains_help_subcommand(&s, "ground"),
        "ground was removed; help was: {s}"
    );
    assert!(
        !contains_help_subcommand(&s, "sync"),
        "sync was removed; help was: {s}"
    );
    assert!(
        contains_help_subcommand(&s, "plan"),
        "expected plan in root help: {s}"
    );
}

#[test]
fn default_prompts_review_plan_has_kpop_and_plan_path_slots() {
    assert!(DEFAULT_PROMPTS_REVIEW_PLAN.contains("{{ kpop }}"));
    assert!(DEFAULT_PROMPTS_REVIEW_PLAN.contains("{{ plan_path }}"));
}

#[test]
fn init_cmd_does_not_reference_grounding_template() {
    let init_src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/init_cmd.rs"));
    assert!(
        !init_src.contains("grounding.md"),
        "init must not install or embed grounding.md"
    );
}

#[test]
fn root_gitignore_ignores_malvin_logs_and_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        common::check_ignored(root, "_malvin/dummy_stamp/plan.md"),
        "expected _malvin/ run dirs to be ignored"
    );
    assert!(
        common::check_ignored(root, "log"),
        "expected root log file to be ignored"
    );
    assert!(
        common::check_ignored(root, "log_2"),
        "expected root log_2 to be ignored"
    );
    assert!(
        common::check_ignored(root, "target/debug/malvin"),
        "expected Rust target/ tree to be ignored"
    );
    assert!(
        !common::check_ignored(root, "README.md"),
        "expected README.md not to be ignored"
    );
}

#[test]
fn init_template_gitignore_is_consistent_with_git_check_ignore() {
    const TEMPLATE: &str = INIT_TEMPLATE_GITIGNORE;
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join(".gitignore"), TEMPLATE).unwrap();
    let st = Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    assert!(st.success(), "git init failed");
    assert!(
        common::check_ignored(tmp.path(), "_malvin/x/plan.md"),
        "template should ignore _malvin/ runs"
    );
    assert!(
        common::check_ignored(tmp.path(), "log"),
        "template should ignore root log"
    );
    assert!(
        common::check_ignored(tmp.path(), "log_2"),
        "template should ignore root log_2"
    );
    assert!(
        common::check_ignored(tmp.path(), "target/release/foo"),
        "template should ignore Rust target/"
    );
    assert!(
        !common::check_ignored(tmp.path(), "src/lib.rs"),
        "template should not ignore normal sources"
    );
    assert!(
        common::check_ignored(tmp.path(), "pkg/__pycache__/x.py"),
        "template should ignore sources under nested __pycache__ dirs (not only *.pyc)"
    );
    assert!(
        common::check_ignored(tmp.path(), "lib/foo.pyc"),
        "template should ignore .pyc via **/*.py[cod]"
    );
}
