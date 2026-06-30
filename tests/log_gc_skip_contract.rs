//! Integration smoke: GC skip for `init`, GC-on for `do` and `code`.

use malvin::output::{format_who_tag_delim, MALVIN_WHO};

mod common;

use std::path::Path;

use common::{malvin_init_output_with_home, test_home_workspace};

const SEED_RUN: &str = "20260101_000000_seedseed";
const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";

fn seed_log_run(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    let seed = common::malvin_run_logs_bucket(work_dir, home).join(SEED_RUN);
    std::fs::create_dir_all(&seed).expect("seed run dir");
    std::fs::write(seed.join("marker.txt"), "seed\n").expect("seed marker");
    seed
}

fn write_gc_config_age_only(home: &Path) {
    std::fs::create_dir_all(home.join(malvin::MALVIN_USER_HOME_DIR)).expect("mkdir .malvin_home");
    std::fs::write(
        home.join(malvin::MALVIN_USER_HOME_DIR).join("config.toml"),
        "[logs]\nmax_count = 0\nmax_age_days = 30\nmax_bytes = \"\"\nmpc = false\n",
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
fn malvin_do_prunes_preexisting_log_dirs() {
    use common::{
        acp_mock_do_streaming_update_js, combined_cli_output, command_output_with_timeout,
        cached_mock_executable, INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT,
    };
    use std::process::Command;

    let (_root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    write_gc_config_age_only(&home);
    let old = seed_old_run(&workspace, &home);

    let mock = cached_mock_executable( &acp_mock_do_streaming_update_js());
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .args(["--no-tee", "do"]);
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.arg("say hi");
    let out =
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin do");
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "malvin do failed: {combined:?}");
    assert!(
        combined.contains("pruned 1 run log(s)"),
        "malvin do must GC before creating run dir: {combined:?}"
    );
    assert!(
        combined.contains(&format_who_tag_delim(MALVIN_WHO)),
        "prune line must use standard malvin logger tag: {combined:?}"
    );
    assert!(!old.exists(), "malvin do must GC aged seeded run dir");
}

#[cfg(unix)]
#[test]
fn malvin_code_artifacts_creation_prunes_preexisting_log_dirs() {
    use common::test_home_workspace;

    let (_root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    write_gc_config_age_only(&home);
    let old = seed_old_run(&workspace, &home);

    malvin::artifacts::create_kpop_run_artifacts("code", Some(&workspace))
        .expect("create code run artifacts");

    assert!(!old.exists(), "code run dir creation must GC aged seeded run dir");
}
