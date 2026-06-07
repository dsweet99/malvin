mod common;

use common::check_ignored;
use std::path::Path;
use std::process::Command;

const INIT_TEMPLATE_GITIGNORE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/default_repo/gitignore"
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
fn help_no_markdown_description_is_disable_styled_markdown() {
    let out = run_root_help_output();
    assert!(
        out.status.success(),
        "help failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    let idx = s
        .lines()
        .position(|line| line.contains("--no-markdown"))
        .unwrap_or_else(|| panic!("expected --no-markdown in root help: {s}"));
    let window = s.lines().skip(idx).take(2).collect::<Vec<_>>().join("\n");
    assert!(
        window.contains("Disable styled markdown"),
        "expected --no-markdown help to say 'Disable styled markdown': {window:?}"
    );
}

#[cfg_attr(unix, test)]
fn help_omits_removed_ground_sync_and_hunt_commands() {
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
        !contains_help_subcommand(&s, "hunt"),
        "hunt was removed; help was: {s}"
    );
    assert!(
        contains_help_subcommand(&s, "plan"),
        "plan subcommand should appear in help; help was: {s}"
    );
}

#[cfg_attr(unix, test)]
fn help_lists_max_acp_retries_with_default_three() {
    let out = run_root_help_output();
    assert!(
        out.status.success(),
        "help failed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert_eq!(help_option_count(&s, "--max-acp-retries"), 1);
    let idx = s
        .lines()
        .position(|line| line.contains("--max-acp-retries"))
        .unwrap_or_else(|| panic!("expected --max-acp-retries in root help: {s}"));
    let window = s.lines().skip(idx).take(2).collect::<Vec<_>>().join("\n");
    assert!(
        window.contains("[default: 3]"),
        "expected --max-acp-retries help to show default 3: {window:?}"
    );
}

#[test]
fn repo_root_gitignore_ignores_malvin_logs_and_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        check_ignored(root, ".malvin/logs/x/plan.md"),
        "repo .gitignore should ignore .malvin/logs/ runs"
    );
    assert!(
        check_ignored(root, "log"),
        "repo .gitignore should ignore root log"
    );
    assert!(
        check_ignored(root, "target/release/foo"),
        "repo .gitignore should ignore Rust target/"
    );
}

#[test]
fn init_template_gitignore_ignores_deepswe_results() {
    assert!(
        INIT_TEMPLATE_GITIGNORE
            .lines()
            .any(|line| line.trim() == "results/"),
        "init template .gitignore should ignore DeepSWE eval artifacts under results/"
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
        check_ignored(tmp.path(), ".malvin/logs/x/plan.md"),
        "template should ignore .malvin/logs/ runs"
    );
    assert!(
        check_ignored(tmp.path(), "log"),
        "template should ignore root log"
    );
    assert!(
        check_ignored(tmp.path(), "log_2"),
        "template should ignore root log_2"
    );
    assert!(
        check_ignored(tmp.path(), "target/release/foo"),
        "template should ignore Rust target/"
    );
    assert!(
        !check_ignored(tmp.path(), "src/lib.rs"),
        "template should not ignore normal sources"
    );
    assert!(
        check_ignored(tmp.path(), "pkg/__pycache__/x.py"),
        "template should ignore sources under nested __pycache__ dirs (not only *.pyc)"
    );
    assert!(
        check_ignored(tmp.path(), "lib/foo.pyc"),
        "template should ignore .pyc via **/*.py[cod]"
    );
}
