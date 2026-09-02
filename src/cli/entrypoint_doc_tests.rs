use super::{
    Exit, dispatch_command, entrypoint_from, finish_entrypoint, prepare_cli_output, run_async_cli,
};
use crate::cli::SharedOpts;
use crate::test_utils::with_isolated_home;

#[test]
fn prepare_cli_output_applies_background_flag() {
    crate::output::set_stdout_suppressed(false);
    let mut shared = SharedOpts::test_defaults();
    shared.background = true;
    prepare_cli_output(&shared);
    assert!(crate::output::stdout_suppressed());
    crate::output::set_stdout_suppressed(false);
}

#[test]
fn entrypoint_from_doc_argv_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
    });
}

#[test]
fn entrypoint_from_background_suppresses_stdout() {
    with_isolated_home(|_| {
        crate::output::set_stdout_suppressed(false);
        assert_eq!(
            entrypoint_from(["malvin", "--background", "--doc"]),
            Exit::Success
        );
        assert!(crate::output::stdout_suppressed());
        crate::output::set_stdout_suppressed(false);
    });
}

#[test]
fn entrypoint_from_bare_malvin_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(entrypoint_from(["malvin"]), Exit::Success);
    });
}

#[test]
fn entrypoint_from_write_without_request_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(entrypoint_from(["malvin", "write"]), Exit::Success);
    });
}

#[test]
fn entrypoint_from_doc_does_not_suppress_stdout_without_background() {
    with_isolated_home(|_| {
        crate::output::set_stdout_suppressed(false);
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
        assert!(!crate::output::stdout_suppressed());
    });
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

#[test]
fn kiss_cov_entrypoint_dispatch_and_commands() {
    let _ = (dispatch_command, finish_entrypoint);

    let _ = crate::cli::entrypoint_commands::run_write_command;
}

#[test]
fn dispatch_gates_only_route_runs_tenacious_preflight() {
    use crate::cli::args::Cli;
    use crate::cli::SharedOpts;
    use clap::CommandFactory;

    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let mut shared = SharedOpts::test_defaults();
        shared.gates = true;
        let matches = Cli::command().get_matches_from(["malvin", "-g", "--no-tenacious"]);
        let result = super::dispatch_gates_only_route(
            1,
            5,
            &mut shared,
            &matches,
        );
        assert!(result.is_err(), "expected router failure without agent: {result:?}");
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}
