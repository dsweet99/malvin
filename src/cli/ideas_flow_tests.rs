#[cfg(test)]
mod ideas_tests {
    use clap::Parser;

    use crate::cli::{Cli, Commands};
    use crate::ideas_flow::{build_ideas_render_context, render_ideas_prompt};
    use crate::prompts::{
        PromptStore, malformed_brace_placeholders, render_mbc2_for_scheduled_kpop_block,
    };

    #[test]
    fn render_ideas_prompt_substitutes_num_ideas_and_user_prompt() {
        let out = render_ideas_prompt(7, "ALPHA_PROMPT").expect("render");
        assert!(out.contains('7'));
        assert!(out.contains("ALPHA_PROMPT"));
        assert!(!out.contains("{{"));
        assert!(malformed_brace_placeholders(&out).is_empty());
    }

    #[test]
    fn render_mbc2_for_scheduled_kpop_block_matches_render_ideas_prompt() {
        let store = PromptStore::default_store();
        let ctx = build_ideas_render_context(3, "BETA");
        let a = render_mbc2_for_scheduled_kpop_block(&store, &ctx).expect("block");
        let b = render_ideas_prompt(3, "BETA").expect("prompt");
        assert_eq!(a, b);
    }

    #[test]
    fn build_ideas_render_context_keys() {
        let ctx = build_ideas_render_context(5, "x");
        assert_eq!(ctx.get("num_ideas").map(String::as_str), Some("5"));
        assert_eq!(ctx.get("user_prompt").map(String::as_str), Some("x"));
    }

    #[test]
    fn cli_accepts_ideas_and_passes_request() {
        let cli = Cli::try_parse_from(["malvin", "invent", "explore edges"]).expect("parse");
        match cli.command {
            Some(Commands::Invent(m)) => {
                assert_eq!(m.request.as_deref(), Some("explore edges"));
                assert_eq!(m.num_ideas, 3);
            }
            _ => panic!("expected Ideas subcommand"),
        }
    }

    #[test]
    fn cli_accepts_ideas_num_ideas() {
        let cli = Cli::try_parse_from(["malvin", "invent", "--num-ideas", "9", "q"]).expect("parse");
        match cli.command {
            Some(Commands::Invent(m)) => {
                assert_eq!(m.num_ideas, 9);
                assert_eq!(m.request.as_deref(), Some("q"));
            }
            _ => panic!("expected Ideas subcommand"),
        }
    }

    #[test]
    fn cli_ideas_doc_parses_without_request() {
        let cli = Cli::try_parse_from(["malvin", "invent", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command.as_ref() {
            Some(Commands::Invent(m)) => assert!(m.request.is_none()),
            _ => panic!("expected Ideas"),
        }
    }
}
