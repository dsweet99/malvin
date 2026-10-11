use std::time::{Duration, Instant};

use super::{
    deliver_finished_dm, dm_hook_config, dm_hook_exec, force_modal_mark, reset_dm_hook_state,
    set_test_dm_hook, set_test_dm_shell, take_test_marks,
};
use crate::output::{DM_END, DM_START, feed_do_dm_stdout_text, set_do_dm_stdout_mode};

struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        reset_dm_hook_state();
        set_do_dm_stdout_mode(false);
    }
}

#[test]
fn on_emit_string_parses_and_other_shapes_do_not_run() {
    let tokens = dm_hook_config::on_emit_tokens("[dm]\non_emit = \"slack send {body}\"\n")
        .expect("string")
        .expect("some");
    assert_eq!(tokens, ["slack", "send", "{body}"]);
    assert!(dm_hook_config::on_emit_tokens("[dm]\non_emit = \"\"\n").unwrap().is_none());
    assert!(dm_hook_config::on_emit_tokens("[dm]\non_emit = []\n").unwrap().is_none());
    assert!(dm_hook_config::on_emit_tokens("mem_limit_gb = 4\n").unwrap().is_none());
    let err = dm_hook_config::on_emit_tokens("[dm]\non_emit = [\"slack\", \"{body}\"]\n")
        .expect_err("array");
    assert!(err.contains("string"), "{err}");
}

#[test]
fn home_config_supplies_the_command_and_workspace_config_does_not() {
    crate::test_support::test_utils::with_isolated_home(|work| {
        let home = crate::malvin_home_config_path();
        std::fs::create_dir_all(home.parent().expect("parent")).expect("mkdir");
        std::fs::write(&home, "[dm]\non_emit = \"echo {body}\"\n").expect("home");
        std::fs::create_dir_all(work.join(".malvin")).expect("ws dir");
        std::fs::write(work.join(".malvin/config.toml"), "[dm]\non_emit = \"evil\"\n")
            .expect("ws");
        let tokens = dm_hook_config::load_home_tokens().expect("tokens");
        assert_eq!(tokens, ["echo", "{body}"]);
    });
}

#[test]
fn streamed_body_is_one_argument_and_indexes_count_finished_messages() {
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    set_test_dm_hook(Some(vec![
        "sh".into(),
        "-c".into(),
        "printf '%s:%s\\n' \"$MALVIN_DM_INDEX\" \"$1\" >> \"$2\"".into(),
        "dm".into(),
        "{body}".into(),
        out.display().to_string(),
    ]));
    set_do_dm_stdout_mode(true);
    feed_do_dm_stdout_text(&format!("{DM_START}\nline—one\n"));
    feed_do_dm_stdout_text("line—two\n");
    feed_do_dm_stdout_text(&format!("{DM_END}\n"));
    feed_do_dm_stdout_text(&format!("{DM_START}\nsecond\n{DM_END}\n"));
    let text = std::fs::read_to_string(&out).expect("out");
    assert_eq!(text, "1:line—one\nline—two\n2:second\n");
}

#[test]
fn shell_redirect_writes_the_body_without_interpreting_it() {
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("dm");
    let pwn = dir.path().join("pwned");
    let spec = format!("echo {{body}} > '{}'", out.display());
    set_test_dm_shell(Some(spec));
    let body = format!("a'b $(echo no) > {}", pwn.display());
    deliver_finished_dm(&body);
    assert_eq!(std::fs::read_to_string(&out).expect("out"), format!("{body}\n"));
    assert!(!pwn.exists(), "shell interpreted the body");
}

fn stdin_carries_the_body_when_the_command_has_no_body_word() {
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    set_test_dm_hook(Some(sh_cat(&out)));
    deliver_finished_dm("via stdin\n");
    assert_eq!(std::fs::read_to_string(&out).expect("out"), "via stdin\n");
}

#[test]
fn a_finished_message_runs_the_hook_when_quiet_mode_is_off() {
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    set_test_dm_hook(Some(sh_cat(&out)));
    set_do_dm_stdout_mode(false);
    feed_do_dm_stdout_text(&format!("{DM_START}\nplain\n{DM_END}\n"));
    assert_eq!(std::fs::read_to_string(&out).expect("out"), "plain");
}

#[test]
fn empty_and_unfinished_messages_do_not_start_the_program() {
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    set_test_dm_hook(Some(sh_cat(&out)));
    set_do_dm_stdout_mode(true);
    feed_do_dm_stdout_text(&format!("{DM_START}\n{DM_END}\n"));
    feed_do_dm_stdout_text(&format!("{DM_START}\nstill open\n"));
    assert!(!out.exists(), "hook ran too early");
}

#[test]
fn failure_is_logged_and_success_stays_off_stderr() {
    let _reset = Reset;
    let _guard = super::super::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().expect("temp");
    let log = dir.path().join("stdout.log");
    crate::output::set_stdout_log_path(Some(log.clone()));
    crate::output::clear_captured_stderr_lines();
    dm_hook_exec::exec_hook(&sh_exit("echo OUTHOOK; echo ERRHOOK >&2; exit 4"), None, secs(2));
    let logged = std::fs::read_to_string(&log).expect("log");
    let err = crate::output::take_captured_stderr_lines();
    crate::output::set_stdout_log_path(None);
    assert!(logged.contains("stdout:\n"), "{logged}");
    assert!(logged.contains("OUTHOOK"), "{logged}");
    assert!(logged.contains("stderr:\n"), "{logged}");
    assert!(logged.contains("ERRHOOK"), "{logged}");
    assert!(logged.contains("exited 4"), "{logged}");
    assert!(err.iter().any(|line| line.contains("exited 4")), "{err:?}");
    crate::output::clear_captured_stderr_lines();
    crate::output::set_stdout_log_path(Some(log.clone()));
    dm_hook_exec::exec_hook(&sh_exit("exit 0"), None, secs(2));
    crate::output::set_stdout_log_path(None);
    let err = crate::output::take_captured_stderr_lines();
    assert!(err.iter().all(|line| !line.contains("dm hook")), "{err:?}");
}

#[test]
fn missing_program_and_timeout_are_reported() {
    let _reset = Reset;
    crate::output::clear_captured_stderr_lines();
    dm_hook_exec::exec_hook(&["malvin-missing-dm-hook".into()], None, secs(2));
    let err = crate::output::take_captured_stderr_lines();
    assert!(
        err.iter().any(|line| line.contains("malvin-missing-dm-hook")),
        "{err:?}"
    );
    let started = Instant::now();
    dm_hook_exec::exec_hook(&sh_exit("sleep 30"), None, Duration::from_millis(200));
    assert!(started.elapsed() < secs(3), "timeout did not return");
    let err = crate::output::take_captured_stderr_lines();
    assert!(err.iter().any(|line| line.contains("timed out")), "{err:?}");
}

#[test]
fn modal_sandbox_emits_a_mark_instead_of_starting_the_program() {
    let _reset = Reset;
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    set_test_dm_hook(Some(sh_cat(&out)));
    force_modal_mark(true);
    deliver_finished_dm("from sandbox");
    assert!(!out.exists(), "sandbox started the program");
    let marks = take_test_marks();
    assert_eq!(marks.len(), 1, "{marks:?}");
    assert_eq!(
        crate::output::parse_dm_emit_mark(&marks[0]),
        Some(Ok("from sandbox".to_string()))
    );
}

#[test]
fn hook_env_records_the_workspace_and_run_directory() {
    let _reset = Reset;
    let _env = crate::test_support::test_utils::test_env_lock();
    let _guard = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("env");
    crate::workspace::run_id::activate_run(dir.path().to_path_buf());
    set_test_dm_hook(Some(vec![
        "sh".into(),
        "-c".into(),
        "printf '%s\\n%s\\n' \"$MALVIN_WORKSPACE\" \"$MALVIN_RUN_DIR\" > \"$1\"".into(),
        "dm".into(),
        out.display().to_string(),
    ]));
    let work = std::env::current_dir().expect("cwd");
    deliver_finished_dm("env");
    crate::workspace::run_id::deactivate_run();
    let text = std::fs::read_to_string(&out).expect("env");
    let mut lines = text.lines();
    assert_eq!(lines.next(), Some(work.to_str().expect("utf8")));
    assert_eq!(lines.next(), Some(dir.path().to_str().expect("utf8")));
}

fn sh_cat(path: &std::path::Path) -> Vec<String> {
    vec![
        "sh".into(),
        "-c".into(),
        "cat > \"$1\"".into(),
        "dm".into(),
        path.display().to_string(),
    ]
}

fn sh_exit(script: &str) -> Vec<String> {
    vec!["sh".into(), "-c".into(), script.into()]
}

fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}
