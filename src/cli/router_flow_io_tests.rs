use clap::Parser;

#[test]
fn cli_accepts_default_route_request() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "route this task"]).expect("parse");
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("route this task"));
    assert!(!cli.shared.gates);
}

#[test]
fn cli_accepts_global_gates_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--gates", "route this task"]).expect("parse");
    assert!(cli.shared.gates);
}

#[test]
fn cli_accepts_short_gates_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "-g", "route this task"]).expect("parse");
    assert!(cli.shared.gates);
}

#[test]
fn cli_accepts_global_creative_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--creative", "route this task"]).expect("parse");
    assert_eq!(cli.shared.creative, Some(1.0));
    assert_eq!(cli.request.as_deref(), Some("route this task"));

    let with_p =
        Cli::try_parse_from(["malvin", "--creative=0.6", "route this task"]).expect("parse");
    assert_eq!(with_p.shared.creative, Some(0.6));
    assert_eq!(with_p.request.as_deref(), Some("route this task"));
}

#[test]
fn cli_accepts_global_no_kpop_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--no-kpop", "route this task"]).expect("parse");
    assert!(cli.shared.no_kpop);
    assert_eq!(cli.request.as_deref(), Some("route this task"));
}

#[test]
fn router_client_uses_router_style_agent_io_not_do_style() {
    use crate::agent_backend::build_agent_backend;
    use crate::cli::{SharedOpts, WorkflowCliOptions};

    let shared = SharedOpts {
        background: false,
        model: crate::model_id::parse_model_id(crate::config::DEFAULT_CLI_MODEL).expect("model"),
        no_force: true,
        no_tenacious: false,
        gates: false,

        quiet: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        git: false,
        creative: None,
        no_kpop: false,
    };
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        shared.acp_stdout_markdown_enabled(),
        "router",
    )
    .expect("backend");
    let io = backend.io;
    assert!(
        !io.raw_output,
        "bare route must use styled logging, not do-style raw_output"
    );
    assert!(io.show_thoughts_on_stdout);
    assert!(io.emit_stdout_markdown);
}

#[test]
fn cli_accepts_clustered_short_gates_and_quiet() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "-gq", "route this task"]).expect("parse -gq");
    assert!(cli.shared.gates, "expected -g in -gq cluster");
    assert!(cli.shared.quiet, "expected -q in -gq cluster");

    let cli = Cli::try_parse_from(["malvin", "-qg", "route this task"]).expect("parse -qg");
    assert!(cli.shared.gates);
    assert!(cli.shared.quiet);
}
