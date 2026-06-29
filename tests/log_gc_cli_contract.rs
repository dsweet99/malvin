//! Integration smoke: `malvin logs status` and `malvin logs gc`.

mod common;

use std::path::Path;
use std::process::Command;

use common::{
    activate_test_home, combined_cli_output, malvin_run_logs_bucket, seed_malvin_config,
    test_home_workspace, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
};

const RUN_OLD_AGE: &str = "20200101_000000_oldrun01";
const RUN_NEW: &str = "20260629_120000_newrun01";

fn write_gc_config_age_only(home: &Path) {
    std::fs::create_dir_all(home.join(malvin::MALVIN_USER_HOME_DIR)).expect("mkdir home");
    std::fs::write(
        home.join(malvin::MALVIN_USER_HOME_DIR).join("config.toml"),
        "[logs]\nmax_count = 0\nmax_age_days = 30\nmax_bytes = \"\"\n",
    )
    .expect("write config");
}

fn seed_old_run(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    let old = malvin_run_logs_bucket(work_dir, home).join(RUN_OLD_AGE);
    std::fs::create_dir_all(&old).expect("seed run dir");
    std::fs::write(old.join("marker.txt"), "seed\n").expect("seed marker");
    old
}

fn run_malvin_logs(args: &[&str], workspace: &Path, home: &Path) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .env("HOME", home)
        .env(malvin::MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION, "1")
        .args(["--no-tee", "logs"]);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin logs")
}

#[cfg(unix)]
#[test]
fn malvin_logs_status_prints_bucket_fields() {
    let (_root, home, workspace) = test_home_workspace();
    activate_test_home(&home);
    seed_malvin_config(
        &workspace,
        "[logs]\nmax_count = 1000\nmax_age_days = 90\nmax_bytes = \"2GiB\"\n",
    );
    let bucket = malvin_run_logs_bucket(&workspace, &home);
    std::fs::create_dir_all(bucket.join(RUN_NEW)).expect("seed run");

    let out = run_malvin_logs(&["status"], &workspace, &home);
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "logs status failed: {combined:?}");
    assert!(combined.contains("bucket:"), "{combined:?}");
    assert!(combined.contains("run count:"), "{combined:?}");
    assert!(combined.contains("total bytes:"), "{combined:?}");
    assert!(combined.contains("oldest run:"), "{combined:?}");
    assert!(combined.contains("newest run:"), "{combined:?}");
    assert!(combined.contains("max_count:"), "{combined:?}");
    assert!(combined.contains("would byte cap prune:"), "{combined:?}");
}

#[cfg(unix)]
#[test]
fn malvin_logs_gc_dry_run_does_not_delete() {
    let (_root, home, workspace) = test_home_workspace();
    activate_test_home(&home);
    write_gc_config_age_only(&home);
    let old = seed_old_run(&workspace, &home);

    let out = run_malvin_logs(&["gc", "--dry-run"], &workspace, &home);
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "logs gc --dry-run failed: {combined:?}");
    assert!(combined.contains("would prune"), "{combined:?}");
    assert!(old.is_dir(), "dry-run must not delete run dirs");
}

#[cfg(unix)]
#[test]
fn malvin_logs_gc_deletes_aged_seed_run() {
    let (_root, home, workspace) = test_home_workspace();
    activate_test_home(&home);
    write_gc_config_age_only(&home);
    let old = seed_old_run(&workspace, &home);

    let out = run_malvin_logs(&["gc"], &workspace, &home);
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "logs gc failed: {combined:?}");
    assert!(combined.contains("pruned 1 run log(s)"), "{combined:?}");
    assert!(!old.exists(), "logs gc must delete aged seeded run dir");
}

#[cfg(unix)]
#[test]
fn malvin_logs_gc_all_buckets_removes_empty_hash_dir() {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    activate_test_home(&home);
    write_gc_config_age_only(&home);

    let empty_bucket = home
        .join(malvin::MALVIN_USER_HOME_DIR)
        .join("logs")
        .join("deadbeefdeadbeef");
    std::fs::create_dir_all(&empty_bucket).expect("mkdir empty bucket");
    assert!(empty_bucket.is_dir());

    let out = run_malvin_logs(&["gc", "--all-buckets"], &workspace, &home);
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "logs gc --all-buckets failed: {combined:?}");
    assert!(
        combined.contains("removed 1 empty log bucket(s)"),
        "{combined:?}"
    );
    assert!(!empty_bucket.exists(), "empty bucket dir must be removed");
}
