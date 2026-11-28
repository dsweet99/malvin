use std::sync::MutexGuard;
use std::time::{Duration, Instant};

use super::{deliver_relayed_direct_message, notify_completed_direct_messages, with_dm_on_emit};
use crate::output::{
    DM_END, DM_START, STDOUT_LOG_TEST_LOCK, clear_captured_stderr_lines, clear_test_marks,
    enable_stdout_capture, force_modal_mark, set_do_dm_stdout_mode, set_stdout_log_path,
    take_captured_stderr_lines, take_captured_stdout, take_test_marks,
};

fn lock_stdout_log() -> MutexGuard<'static, ()> {
    let guard = STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    set_stdout_log_path(None);
    guard
}

fn fenced(body: &str) -> String {
    format!("{DM_START}\n{body}\n{DM_END}\n")
}

fn read(path: &std::path::Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn quiet_mode_delivers_a_dm_once_and_neither_shows_nor_delivers_a_quoted_one() {
    let _log = lock_stdout_log();
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    let reply = format!("{}```\n{}```\n", fenced("real"), fenced("quoted"));
    set_do_dm_stdout_mode(true);
    enable_stdout_capture();
    with_dm_on_emit(&format!("printf '%s\\n' {{body}} >> '{}'", out.display()), None, || {
        crate::backends::bridge_sdk::feed_do_dm_run_result(&reply);
    });
    let shown = crate::ansi_strip::strip_ansi_escapes(&take_captured_stdout());
    set_do_dm_stdout_mode(false);
    assert_eq!(read(&out), "real\n");
    assert!(shown.contains("real"), "{shown:?}");
    assert!(!shown.contains("quoted"), "{shown:?}");
}

#[test]
fn a_shell_template_runs_through_sh_without_warnings() {
    let _log = lock_stdout_log();
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    let template = format!("b={{body}}; printf '%s' \"$b\" > '{}'", out.display());
    clear_captured_stderr_lines();
    with_dm_on_emit(&template, None, || {
        notify_completed_direct_messages(&fenced("two words"));
    });
    let err = take_captured_stderr_lines();
    assert_eq!(read(&out), "two words");
    assert!(err.iter().all(|line| !line.contains("dm")), "{err:?}");
}

#[test]
fn stdin_carries_the_body_when_the_template_has_no_body_placeholder() {
    let _log = lock_stdout_log();
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    with_dm_on_emit(&format!("cat > '{}'", out.display()), None, || {
        notify_completed_direct_messages(&fenced("via stdin"));
    });
    assert_eq!(read(&out), "via stdin");
}

#[test]
fn hook_env_counts_messages_and_names_the_workspace_and_run_directory() {
    let _log = lock_stdout_log();
    let _env = crate::test_support::test_utils::test_env_lock();
    let _run = crate::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::dm_on_emit_run::reset_dm_index_for_test();
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("env");
    let template = format!(
        "printf '%s|%s|%s\\n' \"$MALVIN_DM_INDEX\" \"$MALVIN_WORKSPACE\" \"$MALVIN_RUN_DIR\" >> '{}'",
        out.display()
    );
    crate::workspace::run_id::activate_run(dir.path().to_path_buf());
    with_dm_on_emit(&template, None, || {
        notify_completed_direct_messages(&format!("{}{}", fenced("a"), fenced("b")));
    });
    crate::workspace::run_id::deactivate_run();
    let work = std::env::current_dir().expect("cwd");
    let run = dir.path().display();
    let expected = format!("1|{}|{run}\n2|{}|{run}\n", work.display(), work.display());
    assert_eq!(read(&out), expected);
}

#[cfg(unix)]
#[test]
fn timeout_kills_the_whole_hook_process_group() {
    let dir = tempfile::tempdir().expect("temp");
    let pid_file = dir.path().join("pid");
    let template = format!(
        "/bin/sleep 30 & echo $! > '{}'; wait",
        pid_file.display()
    );
    let started = Instant::now();
    with_dm_on_emit(&template, Some(Duration::from_millis(300)), || {
        notify_completed_direct_messages(&fenced("x"));
    });
    assert!(started.elapsed() < Duration::from_secs(3), "timeout did not return");
    let pid = read(&pid_file).trim().to_string();
    assert!(!pid.is_empty(), "background job did not start");
    assert!(process_gone(&pid), "background sleep {pid} outlived the timeout");
}

#[cfg(unix)]
fn process_gone(pid: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        let stat = read(std::path::Path::new(&format!("/proc/{pid}/stat")));
        let state = stat.rsplit(')').next().and_then(|rest| rest.split_whitespace().next());
        if stat.is_empty() || state == Some("Z") {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn modal_sandbox_publishes_a_mark_and_the_host_runs_the_command_once() {
    let _log = lock_stdout_log();
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("out");
    let template = format!("printf '%s\\n' {{body}} >> '{}'", out.display());
    clear_test_marks();
    force_modal_mark(true);
    with_dm_on_emit(&template, None, || {
        notify_completed_direct_messages(&fenced("from sandbox"));
    });
    force_modal_mark(false);
    let marks = take_test_marks();
    assert!(!out.exists(), "the sandbox ran the command itself");
    assert_eq!(marks.len(), 1, "{marks:?}");
    let body = crate::output::parse_dm_emit_mark(&marks[0])
        .expect("mark")
        .expect("valid");
    with_dm_on_emit(&template, None, || deliver_relayed_direct_message(&body));
    assert_eq!(read(&out), "from sandbox\n");
}
