use clap::{CommandFactory, Parser};

use crate::cli::Cli;
use crate::output::{
    DM_END, DM_START, enable_stdout_capture, feed_do_dm_stdout_text, reset_dm_hook_state,
    set_do_dm_stdout_mode, set_test_dm_hook, take_captured_stdout,
};

#[test]
fn no_emit_disables_both_hooks_and_next_invocation_restores_delivery() {
    let _lock = crate::output::DO_DM_TEST_LOCK.lock().expect("DM lock");
    crate::test_support::test_utils::with_isolated_home(|work| {
        let capture = work.join("deliveries");
        let command = format!("printf '%s' {{body}} >> '{}'", capture.display());
        set_test_dm_hook(Some(vec![
            "tee".into(),
            "-a".into(),
            capture.display().to_string(),
        ]));
        super::with_dm_on_emit(&command, None, || check_invocations(&capture));
        reset_dm_hook_state();
        crate::output::set_dm_on_emit_command(None);
    });
}

fn check_invocations(capture: &std::path::Path) {
    let default = Cli::try_parse_from(["malvin", "task"]).expect("default CLI");
    assert!(!default.shared.no_emit);
    let disabled =
        Cli::try_parse_from(["malvin", "--no-emit", "-q", "task"]).expect("disabled CLI");
    crate::cli::entrypoint::prepare_cli_output(&disabled.shared);
    set_do_dm_stdout_mode(true);
    enable_stdout_capture();
    emit_both("suppressed");
    assert!(!capture.exists(), "--no-emit must prevent both hooks");
    assert!(
        take_captured_stdout().contains("suppressed"),
        "DM stdout must remain visible"
    );
    crate::cli::entrypoint::prepare_cli_output(&default.shared);
    emit_both("delivered");
    assert_eq!(
        std::fs::read_to_string(capture).expect("delivery"),
        "delivereddelivered"
    );
    set_do_dm_stdout_mode(false);
}

fn emit_both(body: &str) {
    let text = format!("{DM_START}\n{body}\n{DM_END}\n");
    feed_do_dm_stdout_text(&text);
    super::notify_completed_direct_messages(&text);
}

#[test]
fn help_explains_no_emit() {
    let help = Cli::command().render_help().to_string();
    assert!(help.contains("--no-emit"), "{help}");
    assert!(help.contains("dm.on_emit"), "{help}");
}
