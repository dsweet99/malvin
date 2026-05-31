//! Integration smoke: GC skip for `do`/`init`, GC-on for `code`.

use malvin::output::{format_who_tag_delim, MALVIN_WHO};

mod common;

use std::path::Path;

use common::{malvin_init_output_with_home, run_do_with_mock, test_home_workspace};

const SEED_RUN: &str = "20260101_000000_seedseed";
const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";

fn seed_log_run(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    let seed = common::malvin_run_logs_bucket(work_dir, home).join(SEED_RUN);
    std::fs::create_dir_all(&seed).expect("seed run dir");
    std::fs::write(seed.join("marker.txt"), "seed\n").expect("seed marker");
    seed
}

fn write_gc_config_age_only(home: &Path) {
    std::fs::create_dir_all(home.join(".malvin")).expect("mkdir .malvin");
    std::fs::write(
        home.join(".malvin/config.toml"),
        "[logs]\nmax_age_days = 30\nmax_bytes = \"\"\n",
    )
    .expect("write config");
}

fn seed_old_run(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    let old = common::malvin_run_logs_bucket(work_dir, home).join(RUN_OLD_AGE);
    std::fs::create_dir_all(&old).expect("seed run dir");
    std::fs::write(old.join("marker.txt"), "seed\n").expect("seed marker");
    old
}

#[test]
fn malvin_init_does_not_prune_preexisting_log_dirs() {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    std::fs::create_dir_all(&home).expect("mkdir home");
    let project = root.path().join("project");
    std::fs::create_dir_all(&project).expect("mkdir project");
    let seed = seed_log_run(&project, &home);
    let out = malvin_init_output_with_home(&project, &home, &["python"]);
    assert!(out.status.success(), "malvin init failed: {out:?}");
    assert!(seed.is_dir(), "init must not GC pre-seeded run log dirs");
    assert!(seed.join("marker.txt").is_file());
}

#[cfg(unix)]
#[test]
fn malvin_do_does_not_prune_preexisting_log_dirs() {
    let (_root, home, workspace) = test_home_workspace();
    let seed = seed_log_run(&workspace, &home);
    let out = run_do_with_mock(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    assert!(seed.is_dir(), "malvin do must not GC pre-seeded run log dirs");
    assert!(seed.join("marker.txt").is_file());
}

#[cfg(unix)]
fn run_malvin_code_in_workspace(
    root: &tempfile::TempDir,
    workspace: &Path,
    home: &Path,
) -> std::process::Output {
    use common::{
        acp_mock_code_kpop_steps_js, bin_path_with_fake_kiss, command_output_with_timeout,
        seed_git_kiss_cargo_gate_workspace, write_mock_executable, MALVIN_TEST_CMD_TIMEOUT,
        workspace_kiss_check_only,
    };
    use std::process::Command;

    seed_git_kiss_cargo_gate_workspace(workspace);
    workspace_kiss_check_only(workspace);
    let path = bin_path_with_fake_kiss(root);
    let mock = root.path().join("mock-agent-acp-code-gc");
    write_mock_executable(&mock, &acp_mock_code_kpop_steps_js());
    command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(workspace)
            .env("HOME", home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", &mock)
            .env("PATH", path)
            .args([
                "--no-tee",
                "code",
                "--max-loops",
                "1",
                "ship it",
            ]),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin code")
}

#[cfg(unix)]
#[test]
fn malvin_code_prunes_preexisting_log_dirs() {
    use common::{combined_cli_output, test_home_workspace};

    let (root, home, workspace) = test_home_workspace();
    write_gc_config_age_only(&home);
    let old = seed_old_run(&workspace, &home);

    let out = run_malvin_code_in_workspace(&root, &workspace, &home);
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "malvin code should succeed with one active run dir: {combined:?}"
    );
    assert!(
        combined.contains("pruned 1 run log(s)"),
        "malvin code must GC before creating run dir: {combined:?}"
    );
    assert!(
        combined.contains(&format_who_tag_delim(MALVIN_WHO)),
        "prune line must use standard malvin logger tag: {combined:?}"
    );
    assert!(!old.exists(), "malvin code must GC aged seeded run dir");
}
