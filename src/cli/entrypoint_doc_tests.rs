use super::{finish_entrypoint, prepare_cli_output, run_async_cli, Exit, entrypoint_from};
use crate::cli::args::GlobalOpts;

#[test]
fn prepare_cli_output_applies_color_and_background_flags() {
    crate::output::set_stdout_suppressed(false);
    prepare_cli_output(&GlobalOpts {
        no_color: false,
        background: true,
    });
    assert!(crate::output::stdout_suppressed());
    crate::output::set_stdout_suppressed(false);
}

#[test]
fn entrypoint_from_doc_argv_exits_success() {
    assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
}

#[test]
fn entrypoint_from_background_suppresses_stdout() {
    crate::output::set_stdout_suppressed(false);
    assert_eq!(entrypoint_from(["malvin", "--background", "--doc"]), Exit::Success);
    assert!(crate::output::stdout_suppressed());
    crate::output::set_stdout_suppressed(false);
}

#[test]
fn entrypoint_from_bare_malvin_exits_success() {
    assert_eq!(entrypoint_from(["malvin"]), Exit::Success);
}

#[test]
fn entrypoint_from_no_color_disables_stdout_color() {
    assert_eq!(entrypoint_from(["malvin", "--no-color", "--doc"]), Exit::Success);
    assert!(!crate::output::stdout_use_color());
}

#[test]
fn entrypoint_from_no_color_and_background_apply_together() {
    crate::output::set_stdout_suppressed(false);
    assert_eq!(
        entrypoint_from(["malvin", "--no-color", "--background", "--doc"]),
        Exit::Success
    );
    assert!(!crate::output::stdout_use_color());
    assert!(crate::output::stdout_suppressed());
    crate::output::set_stdout_suppressed(false);
}

#[test]
fn finish_entrypoint_success_and_failure_paths() {
    use crate::test_stderr_capture::capture_stderr_output;

    assert_eq!(finish_entrypoint(Ok(())), Exit::Success);
    let stderr = capture_stderr_output(|| {
        assert_eq!(finish_entrypoint(Err("boom".into())), Exit::Failure);
    });
    assert!(stderr.contains("boom"), "stderr={stderr:?}");
}

#[test]
fn run_async_cli_runs_immediate_ok_future() {
    assert!(run_async_cli(|| async { Ok(()) }).is_ok());
}
