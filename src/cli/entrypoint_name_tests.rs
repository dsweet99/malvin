use super::{Commands, Exit, command_accepts_session_name, entrypoint_from};
use crate::cli::models_cmd::ModelsArgs;

fn shared_opts_parses_git_flag_default_off() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
    assert!(!cli.shared.git);
}

fn shared_opts_parses_git_flag_on() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--git", "--doc"]).expect("parse");
    assert!(cli.shared.git);
}

fn help_lists_git_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(help.contains("--git"), "help={help}");
}

fn shared_opts_parses_creative_flag_default_off() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
    assert!(!cli.shared.creative);
}

fn shared_opts_parses_creative_flag_on() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"]).expect("parse");
    assert!(cli.shared.creative);
}

fn help_lists_creative_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(help.contains("--creative"), "help={help}");
    assert!(
        help.contains("creative") && help.contains("router_b"),
        "help should mention creative router_b prompt: {help}"
    );
}

fn shared_opts_parses_name_equals_form() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--name=foo", "--doc"]).expect("parse");
    assert_eq!(cli.shared.name.as_deref(), Some("foo"));
}

fn shared_opts_parses_name_space_form() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--name", "foo", "--doc"]).expect("parse");
    assert_eq!(cli.shared.name.as_deref(), Some("foo"));
}

fn help_lists_name_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(help.contains("--name"), "help={help}");
}

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

fn bare_help_does_not_create_name_file() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let probe = crate::name_path("probe");
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe"]),
            Exit::Success
        );
        assert!(!probe.exists(), "bare help must not create name file");
    });
}

fn do_workflow_accepts_session_name_via_entrypoint_rules() {
    use crate::cli::config_defaults::parse_cli_with_config_defaults;

    crate::test_utils::with_isolated_home(|_| {
        let (cli, _) =
            parse_cli_with_config_defaults(["malvin", "--name", "probe", "--do", "say hello"])
                .expect("parse --do");
        assert!(cli.do_workflow);
        assert_eq!(cli.shared.name.as_deref(), Some("probe"));
        assert_eq!(cli.request.as_deref(), Some("say hello"));
    });
}

fn models_command_rejects_session_name() {
    assert!(!command_accepts_session_name(&Commands::Models(
        ModelsArgs::default()
    )));
}

fn gates_only_route_accepts_session_name() {
    use clap::Parser;
    assert!(crate::cli::config_defaults::is_gates_only_route(
        &crate::cli::Cli::try_parse_from(["malvin", "-g"]).expect("parse")
    ));
}

fn write_command_rejects_session_name() {
    use crate::cli::write_flow::WriteArgs;
    assert!(!command_accepts_session_name(&Commands::Write(WriteArgs {
        request: Some("topic".to_string()),
        out_path: "write.tex".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: false,
        out_path_explicit: false,
    }),));
}

fn do_with_name_parses() {
    use crate::cli::config_defaults::parse_cli_with_config_defaults;

    crate::test_utils::with_isolated_home(|_| {
        let (cli, _) =
            parse_cli_with_config_defaults(["malvin", "--name", "probe", "--do", "say hello"])
                .expect("parse --do");
        assert!(cli.do_workflow);
        assert!(cli.command.is_none());
        assert_eq!(cli.request.as_deref(), Some("say hello"));
    });
}

fn models_rejects_name_flag() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|_| {
        crate::cli::error_run_log::clear_command_error_run_dir();
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--name", "probe", "models"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("only supported for"),
            "stderr must reject --name on models; got: {stderr:?}"
        );
    });
}

fn inspire_rejects_name_flag() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|_| {
        crate::cli::error_run_log::clear_command_error_run_dir();
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--name", "probe", "inspire", "topic"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("only supported for"),
            "stderr must reject --name on inspire; got: {stderr:?}"
        );
    });
}

fn write_rejects_name_before_preflight() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|work| {
        crate::cli::error_run_log::clear_command_error_run_dir();
        std::env::set_current_dir(work).expect("chdir");
        let checks = work.join(".malvin/gates");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--name", "probe", "write", "topic"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains("only supported for"),
            "stderr must reject --name on write; got: {stderr:?}"
        );
        assert!(
            !checks.exists(),
            "write --name must reject before writing .malvin/gates"
        );
    });
}

#[test]
fn kiss_bundled_cli_entrypoint_name_tests() {
    shared_opts_parses_git_flag_default_off();
    shared_opts_parses_git_flag_on();
    help_lists_git_flag();
    shared_opts_parses_creative_flag_default_off();
    shared_opts_parses_creative_flag_on();
    help_lists_creative_flag();
    shared_opts_parses_name_equals_form();
    shared_opts_parses_name_space_form();
    help_lists_name_flag();
    doc_does_not_create_name_file();
    bare_help_does_not_create_name_file();
    do_workflow_accepts_session_name_via_entrypoint_rules();
    models_command_rejects_session_name();
    gates_only_route_accepts_session_name();
    write_command_rejects_session_name();
    do_with_name_parses();
    models_rejects_name_flag();
    inspire_rejects_name_flag();
    write_rejects_name_before_preflight();
}
