//! Built-in per-command documentation (`default_prompts/docs/*.md`), embedded at compile time.

use std::io::{self, Write};

use super::Commands;

const MALVIN_OVERVIEW_DOC: &str = include_str!("../../default_prompts/docs/malvin.md");

const fn command_doc_markdown(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Init(_) => include_str!("../../default_prompts/docs/init.md"),
        Commands::Do(_) => include_str!("../../default_prompts/docs/do.md"),
        Commands::Ideas(_) => include_str!("../../default_prompts/docs/ideas.md"),
        Commands::Code(_) => include_str!("../../default_prompts/docs/code.md"),
        Commands::Kpop(_) => include_str!("../../default_prompts/docs/kpop.md"),
        Commands::Bug(_) => include_str!("../../default_prompts/docs/bug.md"),
        Commands::Tidy(_) => include_str!("../../default_prompts/docs/tidy.md"),
        Commands::Plan(_) => include_str!("../../default_prompts/docs/plan.md"),
        Commands::Models(_) => include_str!("../../default_prompts/docs/models.md"),
    }
}

const fn doc_text(command: Option<&Commands>) -> &'static str {
    match command {
        Some(cmd) => command_doc_markdown(cmd),
        None => MALVIN_OVERVIEW_DOC,
    }
}

fn print_doc_to_writer(command: Option<&Commands>, mut out: impl Write) -> Result<(), String> {
    out.write_all(doc_text(command).as_bytes())
        .map_err(|e| format!("stdout: {e}"))?;
    Ok(())
}

pub(crate) fn print_doc(command: Option<&Commands>) -> Result<(), String> {
    print_doc_to_writer(command, io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::cli::args::{Commands, IdeasArgs, KpopArgs};
    use crate::cli::models_cmd::ModelsArgs;
    use clap::Parser;

    fn capture_doc(command: Option<&Commands>) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        print_doc_to_writer(command, &mut buf)?;
        Ok(buf)
    }

    #[test]
    fn subcommand_doc_embeds_have_malvin_heading() {
        let md = command_doc_markdown(&Commands::Models(ModelsArgs {}));
        assert!(md.starts_with("# malvin "));
        let md = command_doc_markdown(&Commands::Kpop(KpopArgs {
            max_hypotheses: 1,
            p_creative: 0.1,
            no_learn: true,
            request: None,
        }));
        assert!(md.starts_with("# malvin "));
        let md = command_doc_markdown(&Commands::Ideas(IdeasArgs {
            num_ideas: 3,
            request: None,
        }));
        assert!(md.starts_with("# malvin ideas"));
    }

    #[test]
    fn print_doc_none_writes_full_malvin_md() {
        let out = capture_doc(None).expect("capture");
        assert_eq!(out.as_slice(), MALVIN_OVERVIEW_DOC.as_bytes());
    }

    #[test]
    fn print_doc_some_writes_subcommand_md() {
        let cmd = Commands::Kpop(KpopArgs {
            max_hypotheses: 1,
            p_creative: 0.1,
            no_learn: true,
            request: None,
        });
        let out = capture_doc(Some(&cmd)).expect("capture");
        assert_eq!(out.as_slice(), command_doc_markdown(&cmd).as_bytes());
        assert!(out.starts_with(b"# malvin kpop"));
    }

    #[test]
    fn top_level_doc_parses_without_subcommand() {
        let cli = Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        assert!(cli.command.is_none());
    }

    #[test]
    fn kpop_doc_parses_without_request_when_doc_flag_set() {
        let cli = Cli::try_parse_from(["malvin", "kpop", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command.as_ref() {
            Some(Commands::Kpop(k)) => assert!(k.request.is_none()),
            _ => panic!("expected Kpop"),
        }
    }

    #[test]
    fn init_doc_parses_without_languages_when_doc_flag_set() {
        let cli = Cli::try_parse_from(["malvin", "init", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command.as_ref() {
            Some(Commands::Init(i)) => assert!(i.languages.is_empty()),
            _ => panic!("expected Init"),
        }
    }
}
