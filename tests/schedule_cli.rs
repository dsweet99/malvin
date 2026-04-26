use std::fs;

#[cfg(all(unix, target_os = "linux"))]
use std::process::Output;

mod common;

#[cfg(all(unix, target_os = "linux"))]
use common::{command_output_with_timeout, MALVIN_TEST_CMD_TIMEOUT};

#[cfg(all(unix, target_os = "linux"))]
fn run_schedule_with_input(input: &str, workers: usize) -> Output {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("jobs.json");
    fs::write(&path, input).expect("write jobs");

    let workers = workers.to_string();
    let input_path = path.to_string_lossy();
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(dir.path())
        .args(["schedule", "--workers", &workers, input_path.as_ref()]);

    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("run schedule cli")
}

#[cfg(all(unix, target_os = "linux"))]
#[test]
fn schedule_success_output_matches_reference() {
    let out = run_schedule_with_input(
        r#"[{"id":"ingest","duration_ms":4,"deps":[]},{"id":"render","duration_ms":2,"deps":["ingest"]},{"id":"notify","duration_ms":1,"deps":["ingest"]},{"id":"archive","duration_ms":1,"deps":["render","notify"]}]"#,
        2,
    );
    assert!(out.status.success(), "expected schedule success");
    assert!(
        String::from_utf8_lossy(&out.stdout).trim_end_matches('\n')
            == r#"[{"job":"ingest","worker":0,"start_ms":0,"end_ms":4},{"job":"notify","worker":1,"start_ms":4,"end_ms":5},{"job":"render","worker":0,"start_ms":4,"end_ms":6},{"job":"archive","worker":0,"start_ms":6,"end_ms":7}]"#
    );
}

#[cfg(all(unix, target_os = "linux"))]
#[test]
fn schedule_cycle_fails_with_one_line_err() {
    let out = run_schedule_with_input(
        r#"[{"id":"a","duration_ms":1,"deps":["c"]},{"id":"b","duration_ms":1,"deps":["a"]},{"id":"c","duration_ms":1,"deps":["b"]}]"#,
        2,
    );
    assert!(!out.status.success(), "expected cycle to fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.starts_with("ERR:"));
}

#[cfg(all(unix, target_os = "linux"))]
#[test]
fn schedule_bad_dependency_fails_with_one_line_err() {
    let out = run_schedule_with_input(
        r#"[{"id":"a","duration_ms":3,"deps":["missing"]}]"#,
        2,
    );
    assert!(!out.status.success(), "expected bad dependency to fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.starts_with("ERR:"));
}
