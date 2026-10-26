use clap::Parser;

#[test]
fn cli_accepts_default_route_request() {
    use crate::cli::Cli;

    let cli = Cli::try_parse_from(["malvin", "route this task"]).expect("parse");
    assert!(cli.command.is_none());
    assert_eq!(
        cli.first_request().map(String::as_str),
        Some("route this task")
    );
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
fn cli_rejects_removed_creative_option() {
    use crate::cli::Cli;

    for argv in [
        ["malvin", "--creative", "route this task"],
        ["malvin", "--creative=0.6", "route this task"],
    ] {
        let err = Cli::try_parse_from(argv).expect_err("removed flag");
        let msg = err.to_string();
        assert!(
            msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--creative"),
            "{argv:?}: {msg}"
        );
    }
}

#[test]
fn cli_rejects_removed_watch_option() {
    use crate::cli::Cli;

    let err = Cli::try_parse_from(["malvin", "--watch", "plan.md"]).expect_err("removed");
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected") || msg.contains("--watch"),
        "{msg}"
    );
}

#[test]
fn router_client_uses_router_style_agent_io_not_do_style() {
    use crate::cli::SharedOpts;
    use malvin::backends::agent_backend::build_agent_backend;

    let shared = SharedOpts {
        model: malvin::config::model_id::parse_model_id(malvin::config::DEFAULT_CLI_MODEL)
            .expect("model")
            .into(),
        no_auto_costs: false,
        max_acp_retries: malvin::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        advice: None,
        credits: false,
        ml: crate::cli::shared_opts::MetaLoopCount::Times(1),
        remote: None,
    };
    let backend = build_agent_backend(
        shared.model.first().clone(),
        shared.max_acp_retries,
        shared.acp_stdout_markdown_enabled(),
    );
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
    assert!(cli.router.dm_stdout_only(), "expected -q in -gq cluster");

    let cli = Cli::try_parse_from(["malvin", "-qg", "route this task"]).expect("parse -qg");
    assert!(cli.router.gates);
    assert!(cli.router.dm_stdout_only());
}
