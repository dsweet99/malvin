#[cfg(test)]
mod inspire_tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};
    use crate::inspire_flow::{
        build_inspire_render_context, render_inspire_prompt, render_inspire_summarize_prompt,
    };
    use crate::prompts::{PromptStore, malformed_brace_placeholders, render_inspire_mbc2_prompt};

    #[test]
    fn render_inspire_prompt_substitutes_user_prompt() {
        let out = render_inspire_prompt("ALPHA_PROMPT").expect("render");
        assert!(out.contains("ALPHA_PROMPT"));
        assert!(!out.contains("{{"));
        assert!(malformed_brace_placeholders(&out).is_empty());
        assert!(out.contains("generate 3"));
    }

    #[test]
    fn render_inspire_summarize_prompt_is_static_body() {
        let out = render_inspire_summarize_prompt().expect("render");
        assert!(!out.contains("{{"));
        assert!(malformed_brace_placeholders(&out).is_empty());
        assert!(
            out.contains("Write a summary of your generated ideas"),
            "inspire_summarize.md body must be rendered: {out}"
        );
        assert!(
            out.to_lowercase().contains("as a dm"),
            "inspire_summarize.md body must ask for a DM: {out}"
        );
        assert!(
            out.contains("MALVIN_DM_START"),
            "inspire_summarize.md must define the DM start marker: {out}"
        );
        assert!(
            out.contains("MALVIN_DM_END"),
            "inspire_summarize.md must define the DM end marker: {out}"
        );
        assert!(
            !out.contains("spoken echo"),
            "inspire_summarize.md must contain coherent instructions: {out}"
        );
    }

    #[test]
    fn render_inspire_mbc2_prompt_matches_render_inspire_prompt() {
        let store = PromptStore::default_store();
        let ctx = build_inspire_render_context("BETA");
        let a = render_inspire_mbc2_prompt(&store, &ctx).expect("block");
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
            model: crate::model_id::parse_model_id(crate::config::DEFAULT_CLI_MODEL)
                .expect("model"),
            no_force: true,
            no_tenacious: false,
            gates: false,

            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            git: false,
            creative: false,
        };
        let backend = crate::cli::build_agent_backend(
            &shared,
            WorkflowCliOptions { force: false },
            shared.acp_stdout_markdown_enabled(),
            "inspire",
        )
        .expect("backend");
        let io = backend.io;
        assert!(
            !io.raw_output,
            "inspire must use styled logging, not do-style raw_output"
        );
        assert!(io.show_thoughts_on_stdout);
        assert!(io.emit_stdout_markdown);
    }

    #[test]
    fn inspire_emit_startup_logs_host_resources() {
        use crate::cli::SharedOpts;
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = match crate::artifacts::create_run_artifacts_from_text("topic", Some(tmp.path())) {
            Ok(artifacts) => artifacts,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem
                ) => {
                    eprintln!("skipping host-resource startup test: artifact root unavailable: {error}");
                    return;
                }
            Err(error) => panic!("art: {error}"),
        };
        let shared = SharedOpts {
            model: crate::model_id::parse_model_id(crate::config::DEFAULT_CLI_MODEL)
                .expect("model"),
            no_force: true,
            no_tenacious: false,
            gates: false,

            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            git: false,
            creative: false,
        };
        crate::cli::run_emit::emit_run_startup_sequence(
            &artifacts,
            crate::cli::run_emit::RunStartupEmitOpts::from_shared(&shared, true),
            "topic",
        )
        .expect("startup");
        let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
        assert!(log.contains("Memory:"));
        assert!(log.contains("CPUs:"));
    }
}
