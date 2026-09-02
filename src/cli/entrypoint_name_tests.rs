use super::{entrypoint_from, Commands, Exit};
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
    assert!(cli.shared.creative.is_none());
}

fn shared_opts_parses_creative_flag_on() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"]).expect("parse");
    assert_eq!(cli.shared.creative, Some(1.0));
}

fn help_lists_creative_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(help.contains("--creative"), "help={help}");
}

fn shared_opts_parses_no_kpop_flag_default_off() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
    assert!(!cli.shared.no_kpop);
}

fn shared_opts_parses_no_kpop_flag_on() {
    use clap::Parser;
    let cli = crate::cli::Cli::try_parse_from(["malvin", "--no-kpop", "--doc"]).expect("parse");
    assert!(cli.shared.no_kpop);
}

fn help_hides_no_kpop_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(
        !help.contains("--no-kpop"),
        "hidden --no-kpop must not appear in help: {help}"
    );
}

fn help_omits_name_flag() {
    use clap::CommandFactory;
    let help = crate::cli::Cli::command().render_help().to_string();
    assert!(
        !help.contains("--name"),
        "help must not list removed --name: {help}"
    );
}

fn doc_does_not_create_name_files() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let root = crate::names_registry_root();
        assert_eq!(entrypoint_from(["malvin", "--doc"]), Exit::Success);
        assert!(
            !root.exists() || std::fs::read_dir(&root).expect("read names").next().is_none(),
            "doc must not create name files"
        );
    });
}

fn bare_help_does_not_create_name_files() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let root = crate::names_registry_root();
        assert_eq!(entrypoint_from(["malvin"]), Exit::Success);
        assert!(
            !root.exists() || std::fs::read_dir(&root).expect("read names").next().is_none(),
            "bare help must not create name files"
        );
    });
}

fn do_workflow_parses_without_name_flag() {
    use crate::cli::config_defaults::parse_cli_with_config_defaults;

    crate::test_utils::with_isolated_home(|_| {
        let (cli, _) =
            parse_cli_with_config_defaults(["malvin", "--do", "say hello"]).expect("parse --do");
        assert!(cli.do_workflow);
        assert_eq!(cli.request.as_deref(), Some("say hello"));
        assert!(cli.command.is_none());
    });
}

fn gates_only_route_needs_session() {
    use clap::Parser;
    assert!(crate::cli::config_defaults::is_gates_only_route(
        &crate::cli::Cli::try_parse_from(["malvin", "-g"]).expect("parse")
    ));
}

fn write_command_is_not_gates_only() {
    use crate::cli::write_flow::WriteArgs;
    let _ = Commands::Write(WriteArgs {
        shared: crate::cli::SharedOpts::test_defaults(),
        request: Some("topic".to_string()),
        out_path: "write.tex".to_string(),
        max_loops: 1,
        max_hypotheses: 5,
        tenacious: false,
        out_path_explicit: false,
    });
    let _ = ModelsArgs::default();
}

fn name_flag_is_rejected_by_clap() {
    use clap::Parser;
    let err = crate::cli::Cli::try_parse_from(["malvin", "--name", "probe", "--doc"])
        .expect_err("--name must be unknown");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--name"),
        "clap must reject --name; got: {msg}"
    );
}

#[test]
fn kiss_bundled_cli_entrypoint_name_tests() {
    shared_opts_parses_git_flag_default_off();
    shared_opts_parses_git_flag_on();
    help_lists_git_flag();
    shared_opts_parses_creative_flag_default_off();
    shared_opts_parses_creative_flag_on();
    help_lists_creative_flag();
    shared_opts_parses_no_kpop_flag_default_off();
    shared_opts_parses_no_kpop_flag_on();
    help_hides_no_kpop_flag();
    help_omits_name_flag();
    doc_does_not_create_name_files();
    bare_help_does_not_create_name_files();
    do_workflow_parses_without_name_flag();
    gates_only_route_needs_session();
    write_command_is_not_gates_only();
    name_flag_is_rejected_by_clap();
}
