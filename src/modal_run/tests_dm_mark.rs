use super::remote_output::{RemoteLines, Stream};

#[test]
fn relay_runs_the_hook_from_the_mark_and_keeps_the_mark_off_stdout() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_dm_hook_state();
    let dir = tempfile::tempdir().expect("temp");
    let out = dir.path().join("body");
    crate::output::set_test_dm_hook(Some(vec![
        "sh".into(),
        "-c".into(),
        "cat > \"$1\"".into(),
        "dm".into(),
        out.display().to_string(),
    ]));
    crate::output::set_do_dm_stdout_mode(true);
    crate::output::enable_stdout_capture();
    crate::output::clear_captured_stderr_lines();
    let mut lines = RemoteLines::default();
    let mark = crate::output::mark_line("from modal");
    lines.push(Stream::Stdout, &format!("visible\n{mark}\n"));
    lines.push(Stream::Stdout, "__MALVIN_DM_EMIT__zz\n");
    let shown = crate::ansi_strip::strip_ansi_escapes(&crate::output::take_captured_stdout());
    let err = crate::output::take_captured_stderr_lines();
    crate::output::set_do_dm_stdout_mode(false);
    crate::output::reset_dm_hook_state();
    assert_eq!(shown, "visible");
    assert_eq!(std::fs::read_to_string(out).expect("body"), "from modal");
    assert!(
        err.iter()
            .any(|line| line.contains("broken end-of-DM mark")),
        "{err:?}"
    );
}
