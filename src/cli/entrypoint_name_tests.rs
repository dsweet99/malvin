use super::{
    command_accepts_session_name, unsupported_name_error, Commands, Exit, entrypoint_from,
};
use crate::cli::args_bug_kpop::KpopArgs;
use crate::cli::models_cmd::ModelsArgs;

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

#[test]
fn bare_kpop_command_accepts_session_name_when_bare_invoke() {
    assert!(command_accepts_session_name(
        &Commands::Kpop(KpopArgs {
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: false,
            request: Some("task".into()),
        }),
        true
    ));
}

#[test]
fn explicit_kpop_subcommand_rejects_session_name() {
    assert!(!command_accepts_session_name(
        &Commands::Kpop(KpopArgs {
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: false,
            request: Some("task".into()),
        }),
        false
    ));
    assert!(unsupported_name_error(
        &Commands::Kpop(KpopArgs {
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: false,
            request: Some("task".into()),
        }),
        false
    )
    .is_some());
}

#[test]
fn models_command_rejects_session_name() {
    assert!(!command_accepts_session_name(&Commands::Models(ModelsArgs {}), false));
}

#[test]
fn bare_request_resolves_to_kpop_that_accepts_session_name() {
    use crate::cli::config_defaults::parse_cli_with_config_defaults;

    let (cli, _) =
        parse_cli_with_config_defaults(["malvin", "--name", "probe", "investigate cache"])
            .expect("parse bare kpop");
    let command = cli.command.expect("bare request resolves to subcommand");
    assert!(cli.bare_request.is_some());
    assert!(command_accepts_session_name(&command, true));
}

#[test]
fn models_rejects_name_flag() {
    use crate::test_stderr_capture::capture_stderr_output;

    let stderr = capture_stderr_output(|| {
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "models"]),
            Exit::Failure
        );
    });
    assert!(
        stderr.contains("only supported for bare"),
        "stderr must reject --name on models; got: {stderr:?}"
    );
}

#[test]
fn init_rejects_name_flag() {
    use crate::test_stderr_capture::capture_stderr_output;

    let stderr = capture_stderr_output(|| {
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "init"]),
            Exit::Failure
        );
    });
    assert!(
        stderr.contains("only supported for bare"),
        "stderr must reject --name on init; got: {stderr:?}"
    );
}

#[cfg(unix)]
#[test]
fn bare_kpop_duplicate_name_exits_failure() {
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
            entrypoint_from(["malvin", "--name", "probe", "investigate cache"]),
            Exit::Failure
        );
        let _ = child.kill();
        let _ = child.wait();
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
            entrypoint_from(["malvin", "--name", "probe", "plan", "plan.md"]),
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
                entrypoint_from(["malvin", "--background", "--name", "probe", "plan", "plan.md"]),
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
