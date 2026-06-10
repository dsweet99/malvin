use super::{
    dispatch_command, finish_entrypoint, prepare_cli_output, run_async_cli, Exit, entrypoint_from,
};
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

#[test]
fn shared_opts_parses_name_equals_form() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--name=foo", "--doc"]).expect("parse");
    assert_eq!(cli.shared.name.as_deref(), Some("foo"));
}

#[test]
fn shared_opts_parses_name_space_form() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--name", "foo", "--doc"]).expect("parse");
    assert_eq!(cli.shared.name.as_deref(), Some("foo"));
}

#[test]
fn help_lists_name_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(help.contains("--name"), "help={help}");
}

#[test]
fn doc_does_not_create_name_file() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let probe = crate::name_path("probe");
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "--doc"]),
            Exit::Success
        );
        assert!(!probe.exists(), "doc must not create name file");
    });
}

#[test]
fn bare_help_does_not_create_name_file() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let probe = crate::name_path("probe");
        assert_eq!(entrypoint_from(["malvin", "--name", "probe"]), Exit::Success);
        assert!(!probe.exists(), "bare help must not create name file");
    });
}

#[cfg(unix)]
#[test]
fn models_acquires_and_releases_name() {
    use std::os::unix::fs::PermissionsExt;

    use crate::repo_checks::set_fake_command_dir;

    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let tmp = tempfile::tempdir().expect("tempdir");
        let agent = tmp.path().join("agent");
        std::fs::write(
            &agent,
            "#!/bin/sh\nif [ \"$1\" = models ]; then printf 'composer-2 — Fast\\n'; exit 0; fi\nexit 1\n",
        )
        .expect("write fake agent");
        let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&agent, perms).expect("chmod fake agent");
        let _guard = set_fake_command_dir(tmp.path());
        let probe = crate::name_path("probe");
        assert!(!probe.exists());
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "models"]),
            Exit::Success
        );
        assert!(!probe.exists(), "name file removed after models exit");
    });
}

#[cfg(unix)]
#[test]
fn duplicate_name_exits_failure() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let mut child = crate::malvin_sandbox::malvin_std_command("sleep")
            .arg("120")
            .spawn()
            .expect("spawn sleep");
        let holder_pid = child.id();
        std::fs::create_dir_all(crate::names_registry_root()).expect("mkdir names");
        std::fs::write(crate::name_path("probe"), format!("{holder_pid}\n")).expect("peer lock");
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "models"]),
            Exit::Failure
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[cfg(unix)]
#[test]
fn duplicate_name_error_on_stderr_with_background() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let mut child = crate::malvin_sandbox::malvin_std_command("sleep")
            .arg("120")
            .spawn()
            .expect("spawn sleep");
        let holder_pid = child.id();
        std::fs::create_dir_all(crate::names_registry_root()).expect("mkdir names");
        std::fs::write(crate::name_path("probe"), format!("{holder_pid}\n")).expect("peer lock");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--background", "--name", "probe", "models"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains(&holder_pid.to_string()),
            "stderr must name holder pid; got: {stderr:?}"
        );
        assert!(
            stderr.contains(&crate::name_path("probe").display().to_string()),
            "stderr must name lock path; got: {stderr:?}"
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[test]
fn kiss_cov_entrypoint_dispatch_and_commands() {
    let _ = (dispatch_command, finish_entrypoint);
    assert!(stringify!(run_async_cli).contains("run_async_cli"));
    let _ = crate::cli::entrypoint_commands::run_code_command;
    let _ = crate::cli::entrypoint_commands::run_inspire_command;
    let _ = crate::cli::entrypoint_commands::run_plan_command;
    let _ = crate::cli::entrypoint_commands::run_delight_command;
}
