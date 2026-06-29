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
    assert!(!command_accepts_session_name(&Commands::Models(ModelsArgs { mini: false }), false));
}

#[test]
fn delight_command_accepts_session_name() {
    use crate::cli::delight_flow::DelightArgs;
    assert!(command_accepts_session_name(
        &Commands::Delight(DelightArgs {
            guidance: None,
            out_path: "pitch.md".to_string(),
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: false,
        }),
        false
    ));
}

#[test]
fn explain_command_rejects_session_name() {
    use crate::cli::explain_flow::ExplainArgs;
    assert!(!command_accepts_session_name(
        &Commands::Explain(ExplainArgs {
            request: Some("topic".to_string()),
            out_path: "explain.tex".to_string(),
            max_loops: 1,
            max_hypotheses: 5,
            tenacious: false,
            out_path_explicit: false,
        }),
        false
    ));
}

#[test]
fn bare_request_resolves_to_kpop_that_accepts_session_name() {
    use crate::cli::config_defaults::parse_cli_with_config_defaults;

    let (cli, _) =
        parse_cli_with_config_defaults(["malvin", "--name", "probe", "investigate cache"])
            .expect("parse bare kpop");
    let command = cli.command.expect("bare request resolves to subcommand");
    assert!(cli.bare_args.len() == 1);
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

#[test]
fn explain_rejects_name_before_preflight() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|work| {
        std::env::set_current_dir(work).expect("chdir");
        let checks = work.join(".malvin/checks");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--name", "probe", "explain", "topic"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("only supported for bare"),
            "stderr must reject --name on explain; got: {stderr:?}"
        );
        assert!(
            !checks.exists(),
            "explain --name must reject before writing .malvin/checks"
        );
    });
}

#[test]
fn revise_rejects_name_before_preflight() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|work| {
        std::env::set_current_dir(work).expect("chdir");
        let checks = work.join(".malvin/checks");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--name", "probe", "revise", "doc.md"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("only supported for bare"),
            "stderr must reject --name on revise; got: {stderr:?}"
        );
        assert!(
            !checks.exists(),
            "revise --name must reject before writing .malvin/checks"
        );
    });
}
