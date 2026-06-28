#[cfg(test)]
mod inspire_tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};
    use crate::inspire_flow::{build_inspire_render_context, render_inspire_prompt};
    use crate::prompts::{
        PromptStore, malformed_brace_placeholders, render_mbc2_for_scheduled_kpop_block,
    };

    #[test]
    fn render_inspire_prompt_substitutes_user_prompt() {
        let out = render_inspire_prompt("ALPHA_PROMPT").expect("render");
        assert!(out.contains("ALPHA_PROMPT"));
        assert!(!out.contains("{{"));
        assert!(malformed_brace_placeholders(&out).is_empty());
        assert!(out.contains("generate 3"));
    }

    #[test]
    fn render_mbc2_for_scheduled_kpop_block_matches_render_inspire_prompt() {
        let store = PromptStore::default_store();
        let ctx = build_inspire_render_context("BETA");
        let a = render_mbc2_for_scheduled_kpop_block(&store, &ctx).expect("block");
        let b = render_inspire_prompt("BETA").expect("prompt");
        assert_eq!(a, b);
    }

    #[test]
    fn build_inspire_render_context_keys() {
        let ctx = build_inspire_render_context("x");
        assert_eq!(ctx.get("user_prompt").map(String::as_str), Some("x"));
        assert!(!ctx.contains_key("num_ideas"));
    }

    #[test]
    fn cli_accepts_inspire_and_passes_request() {
        let cli = Cli::try_parse_from(["malvin", "inspire", "explore edges"]).expect("parse");
        match cli.command {
            Some(Commands::Inspire(m)) => {
                assert_eq!(m.request.as_deref(), Some("explore edges"));
            }
            _ => panic!("expected Inspire subcommand"),
        }
    }

    #[test]
    fn cli_rejects_removed_num_ideas_flag() {
        let err = Cli::try_parse_from(["malvin", "inspire", "--num-ideas", "9", "q"])
            .expect_err("removed flag should fail");
        assert!(err.to_string().contains("num-ideas"));
    }

    #[test]
    fn cli_inspire_doc_parses_without_request() {
        let cli = Cli::try_parse_from(["malvin", "inspire", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command.as_ref() {
            Some(Commands::Inspire(m)) => assert!(m.request.is_none()),
            _ => panic!("expected Inspire"),
        }
    }

    #[test]
    fn inspire_client_uses_styled_agent_io_not_raw_do_style() {
        use crate::cli::{SharedOpts, WorkflowCliOptions};
        let shared = SharedOpts {
            model: crate::config::DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tenacious: false,
            no_tee: true,
            no_markdown: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            mini: false,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        };
        let client = crate::cli::build_agent(
            &shared,
            WorkflowCliOptions { force: false },
            shared.acp_stdout_markdown_enabled(),
        );
        assert!(
            !client.io.raw_output,
            "inspire must use styled logging, not do-style raw_output"
        );
        assert!(client.io.show_thoughts_on_stdout);
        assert!(client.io.emit_stdout_markdown);
    }

    #[test]
    fn inspire_emit_startup_logs_host_resources() {
        use crate::cli::SharedOpts;
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_run_artifacts_from_text("topic", Some(tmp.path()))
                .expect("art");
        let shared = SharedOpts {
            model: crate::config::DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tenacious: false,
            no_tee: true,
            no_markdown: true,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            mini: false,
            mini_max_bash_turns: 32,
            mini_max_http_turns: 32,
            mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        };
        crate::cli::run_emit::emit_run_startup_sequence(
            &artifacts,
            crate::cli::run_emit::RunStartupEmitOpts {
                tee_stdout: shared.tee_startup_stdout(),
                host_resources: true,
            },
            "topic",
        )
        .expect("startup");
        let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
        assert!(log.contains("Memory:"));
        assert!(log.contains("CPUs:"));
    }
}
