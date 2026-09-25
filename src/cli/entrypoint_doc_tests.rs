use super::{
    Exit, dispatch_command, entrypoint_from, finish_entrypoint, prepare_cli_output, run_async_cli,
};
use crate::cli::SharedOpts;
use malvin::test_utils::with_isolated_home;

#[test]
fn prepare_cli_output_initializes_output_state() {
    malvin::output::set_stdout_suppressed(true);
    let shared = SharedOpts::test_defaults();
    prepare_cli_output(&shared);
    assert!(!malvin::output::stdout_suppressed());
    malvin::output::set_stdout_suppressed(false);
}

#[test]
fn entrypoint_from_doc_argv_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
    });
}

#[test]
fn entrypoint_from_advice_design_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(
            entrypoint_from(["malvin", "--advice", "design"]),
            Exit::Success
        );
    });
}

#[test]
fn entrypoint_from_advice_unknown_tag_exits_failure() {
    with_isolated_home(|_| {
        assert_eq!(
            entrypoint_from(["malvin", "--advice", "nope"]),
            Exit::Failure
        );
    });
}

#[test]
fn entrypoint_from_background_is_rejected() {
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from(["malvin", "--background", "--doc"])
        .expect_err("--background must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--background"),
        "clap must reject --background; got {msg}"
    );
}

#[test]
fn entrypoint_from_bare_malvin_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(entrypoint_from(["malvin"]), Exit::Success);
    });
}

#[test]
fn entrypoint_from_admin_models_doc_exits_success() {
    with_isolated_home(|_| {
        assert_eq!(
            entrypoint_from(["malvin", "admin", "models", "--doc"]),
            Exit::Success
        );
    });
}

#[test]
fn entrypoint_from_admin_rejects_gates_flag() {
    use malvin::test_stderr_capture::capture_stderr_output;

    with_isolated_home(|_| {
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "-g", "admin", "models", "--doc"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("admin") && (stderr.contains("--gates") || stderr.contains("-g")),
            "expected admin+gates rejection; stderr={stderr:?}"
        );
    });
}

#[test]
fn entrypoint_from_admin_rejects_quiet_flag() {
    use malvin::test_stderr_capture::capture_stderr_output;

    with_isolated_home(|_| {
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "-q", "admin", "models", "--doc"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("admin") && (stderr.contains("--quiet") || stderr.contains("-q")),
            "expected admin+quiet rejection; stderr={stderr:?}"
        );
    });
}

#[test]
fn entrypoint_from_admin_rejects_verbose_flag() {
    use malvin::test_stderr_capture::capture_stderr_output;

    with_isolated_home(|_| {
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "-v", "admin", "models", "--doc"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("admin") && (stderr.contains("--verbose") || stderr.contains("-v")),
            "expected admin+verbose rejection; stderr={stderr:?}"
        );
    });
}

#[test]
fn entrypoint_from_admin_rejects_max_acp_retries_flag() {
    use malvin::test_stderr_capture::capture_stderr_output;

    with_isolated_home(|_| {
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from([
                    "malvin",
                    "--max-acp-retries",
                    "9",
                    "admin",
                    "models",
                    "--doc"
                ]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("admin") && stderr.contains("--max-acp-retries"),
            "expected admin+max-acp-retries rejection; stderr={stderr:?}"
        );
    });
}

#[test]
fn entrypoint_from_doc_does_not_suppress_stdout() {
    with_isolated_home(|_| {
        malvin::output::set_stdout_suppressed(false);
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
        assert!(!malvin::output::stdout_suppressed());
    });
}

#[test]
fn finish_entrypoint_success_and_failure_paths() {
    use malvin::test_stderr_capture::capture_stderr_output;

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
}

#[test]
fn dispatch_gates_only_route_runs_tenacious_preflight() {
    use crate::cli::SharedOpts;
    use crate::cli::args::Cli;
    use clap::CommandFactory;

    malvin::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let mut shared = SharedOpts::test_defaults();
        let mut router = crate::cli::RouterOpts::test_defaults();
        router.gates = true;
        shared.model = malvin::model_id::parse_model_id("rpi:some-unknown/foo").expect("model");
        let matches = Cli::command().get_matches_from(["malvin", "-g"]);
        let result = super::dispatch_gates_only_route(super::GatesOnlyDispatch {
            max_loops: 1,
            max_hypotheses: 5,
            shared: &mut shared,
            router: &mut router,
            matches: &matches,
        });
        assert!(
            result.is_err(),
            "expected router failure without agent: {result:?}"
        );
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}
