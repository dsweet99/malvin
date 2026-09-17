use clap::Parser;

#[test]
fn cli_accepts_default_route_request() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "route this task"]).expect("parse");
    assert!(cli.command.is_none());
    assert_eq!(cli.first_request().map(String::as_str), Some("route this task"));
    assert!(!cli.router.gates);
}

#[test]
fn cli_accepts_global_gates_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--gates", "route this task"]).expect("parse");
    assert!(cli.router.gates);
}

#[test]
fn cli_accepts_short_gates_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "-g", "route this task"]).expect("parse");
    assert!(cli.router.gates);
}

#[test]
fn cli_accepts_global_creative_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--creative", "route this task"]).expect("parse");
    assert_eq!(cli.router.creative, Some(1.0));
    assert_eq!(cli.first_request().map(String::as_str), Some("route this task"));

    let with_p =
        Cli::try_parse_from(["malvin", "--creative=0.6", "route this task"]).expect("parse");
    assert_eq!(with_p.router.creative, Some(0.6));
    assert_eq!(with_p.first_request().map(String::as_str), Some("route this task"));
}

#[test]
fn cli_accepts_watch_option() {
    use crate::cli::Cli;

    let off = Cli::try_parse_from(["malvin", "route this task"]).expect("parse");
    assert!(!off.router.watch);

    let on = Cli::try_parse_from(["malvin", "--watch", "plan.md"]).expect("parse");
    assert!(on.router.watch);
    assert_eq!(on.first_request().map(String::as_str), Some("plan.md"));

    assert!(
        Cli::try_parse_from(["malvin", "--do", "--watch", "plan.md"]).is_err(),
        "--watch conflicts with --do"
    );
}

#[test]
fn cli_accepts_global_no_kpop_option() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "--no-kpop", "route this task"]).expect("parse");
    assert!(cli.router.no_kpop);
    assert_eq!(cli.first_request().map(String::as_str), Some("route this task"));
}

#[test]
fn router_client_uses_router_style_agent_io_not_do_style() {
    use crate::cli::SharedOpts;
    use malvin::agent_backend::build_agent_backend;

    let shared = SharedOpts {
        model: malvin::model_id::parse_model_id(malvin::config::DEFAULT_CLI_MODEL).expect("model"),
        verbose: false,
        max_acp_retries: malvin::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
    };
    let backend = build_agent_backend(
        shared.model.clone(),
        shared.max_acp_retries,
        shared.acp_stdout_markdown_enabled(),
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
    assert!(cli.router.gates, "expected -g in -gq cluster");
    assert!(cli.router.quiet, "expected -q in -gq cluster");

    let cli = Cli::try_parse_from(["malvin", "-qg", "route this task"]).expect("parse -qg");
    assert!(cli.router.gates);
    assert!(cli.router.quiet);
}
