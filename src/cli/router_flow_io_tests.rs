//! Agent-IO style checks for router backends.

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
fn router_client_uses_kpop_style_agent_io_not_do_style() {
    use crate::agent_backend::build_agent_backend;
    use crate::cli::{SharedOpts, WorkflowCliOptions};

    let shared = SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tenacious: false,
        gates: false,
        no_tee: true,
        no_markdown: false,
        verbose: false,
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    };
    let backend = build_agent_backend(
        &shared,
        WorkflowCliOptions { force: false },
        shared.acp_stdout_markdown_enabled(),
        "router",
    )
    .expect("backend");
    let io = match backend {
        crate::agent_backend::AgentBackend::Acp(c) => c.io,
        crate::agent_backend::AgentBackend::Mini(c) => c.io,
    };
    assert!(
        !io.raw_output,
        "bare route must use styled logging, not do-style raw_output"
    );
    assert!(io.show_thoughts_on_stdout);
    assert!(io.emit_stdout_markdown);
}

#[test]
fn openrouter_router_client_is_mini_with_styled_not_raw_output() {
    use crate::agent_backend::build_agent_backend;
    use crate::cli::{SharedOpts, WorkflowCliOptions};

    crate::acp::with_env(
        "OPENROUTER_API_KEY",
        Some("sk-test-openrouter-router-io"),
        || {
            let shared = SharedOpts {
                model: "openrouter:org/model".into(),
                no_force: true,
                no_tenacious: false,
                gates: false,
                no_tee: true,
                no_markdown: false,
                verbose: false,
                max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
                doc: false,
                name: None,
                mini_max_bash_turns: 32,
                mini_max_http_turns: 32,
                mini_max_bash_execs: 128,
                mini_max_http_retries: 0,
                mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
                mini_max_gate_retries: 0,
                mini_max_shrink_passes: 0,
                no_download: false,
                git: false,
            };
            let backend = build_agent_backend(
                &shared,
                WorkflowCliOptions { force: false },
                shared.acp_stdout_markdown_enabled(),
                "router",
            )
            .expect("openrouter mini backend");
            match backend {
                crate::agent_backend::AgentBackend::Mini(c) => {
                    assert!(
                        !c.io.raw_output,
                        "openrouter router must not use do-style raw_output"
                    );
                    assert!(c.io.show_thoughts_on_stdout);
                    assert!(c.io.emit_stdout_markdown);
                }
                crate::agent_backend::AgentBackend::Acp(_) => {
                    panic!("openrouter: model must select Mini backend");
                }
            }
        },
    );
}
