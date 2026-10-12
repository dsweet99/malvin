use std::time::Duration;

use super::{
    dm_on_emit_command_for_test, notify_completed_direct_messages, render_on_emit, store_command,
    with_config_lock, with_dm_on_emit,
};
use crate::output::{DM_END, DM_START, clear_captured_stderr_lines, take_captured_stderr_lines};

fn fenced(body: &str) -> String {
    format!("{DM_START}\n{body}\n{DM_END}\n")
}

fn capture_template(path: &std::path::Path) -> String {
    format!("printf '%s\\n' {{body}} >> '{}'", path.display())
}

#[test]
fn blank_on_emit_is_unset_and_a_command_is_kept() {
    with_config_lock(|| {
        store_command(Some("  ".to_string()));
        assert_eq!(dm_on_emit_command_for_test(), None);
        store_command(Some("printf %s {body}".to_string()));
        assert_eq!(
            dm_on_emit_command_for_test().as_deref(),
            Some("printf %s {body}")
        );
    });
}

#[test]
fn render_replaces_quoted_placeholder_and_escapes_apostrophes() {
    let template = "curl -H 'Title: malvin' -d '{body}' 'https://example.test/topic'";
    let rendered = render_on_emit(template, "it's\nline");
    assert_eq!(
        rendered,
        "curl -H 'Title: malvin' -d 'it'\\''s\nline' 'https://example.test/topic'"
    );
    assert_eq!(render_on_emit("echo {body}", "a b"), "echo 'a b'");
    assert_eq!(render_on_emit("echo \"{body}\"", "a b"), "echo 'a b'");
}

#[test]
fn notify_writes_one_body_per_completed_message() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("dm.txt");
    with_dm_on_emit(&capture_template(&path), None, || {
        notify_completed_direct_messages(&format!("{DM_START}\npartial\n"));
        notify_completed_direct_messages("noise\n");
        notify_completed_direct_messages(&fenced("it's done"));
        notify_completed_direct_messages(&fenced("second"));
    });
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "it's done\nsecond\n"
    );
}

#[test]
fn notify_runs_when_stdout_dm_mode_is_off() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("off.txt");
    crate::output::set_do_dm_stdout_mode(false);
    with_dm_on_emit(&capture_template(&path), None, || {
        crate::backends::bridge_sdk::feed_do_dm_run_result(&fenced("from-result"));
    });
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "from-result\n"
    );
}

#[test]
fn notify_warns_when_command_fails_or_times_out() {
    clear_captured_stderr_lines();
    with_dm_on_emit("exit 3", None, || {
        notify_completed_direct_messages(&fenced("x"));
    });
    let failed = take_captured_stderr_lines();
    assert!(
        failed.iter().any(|line| line.contains("dm.on_emit exited")),
        "{failed:?}"
    );
    clear_captured_stderr_lines();
    with_dm_on_emit("sleep 30", Some(Duration::from_millis(200)), || {
        notify_completed_direct_messages(&fenced("x"));
    });
    let timed_out = take_captured_stderr_lines();
    assert!(
        timed_out
            .iter()
            .any(|line| line.contains("dm.on_emit timed out")),
        "{timed_out:?}"
    );
}
