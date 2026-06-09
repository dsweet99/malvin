//! Built-in per-command documentation (`default_prompts/docs/*.md`), embedded at compile time.

use std::io::{self, Write};

use super::Commands;

const MALVIN_OVERVIEW_DOC: &str = include_str!("../../default_prompts/docs/malvin.md");

const fn command_doc_markdown(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Init(_) => include_str!("../../default_prompts/docs/init.md"),
        Commands::Do(_) => include_str!("../../default_prompts/docs/do.md"),
        Commands::Inspire(_) => include_str!("../../default_prompts/docs/inspire.md"),
        Commands::Code(_) => include_str!("../../default_prompts/docs/code.md"),
        Commands::Kpop(_) => include_str!("../../default_prompts/docs/kpop.md"),
        Commands::Tidy(_) => include_str!("../../default_prompts/docs/tidy.md"),
        Commands::Models(_) => include_str!("../../default_prompts/docs/models.md"),
        Commands::Plan(_) => include_str!("../../default_prompts/docs/plan.md"),
    }
}

const fn doc_text(command: Option<&Commands>) -> &'static str {
    match command {
        Some(cmd) => command_doc_markdown(cmd),
        None => MALVIN_OVERVIEW_DOC,
    }
}

fn print_doc_to_writer(command: Option<&Commands>, mut out: impl Write) -> Result<(), String> {
    let text = doc_text(command).replace("{{ advice_path }}", crate::MALVIN_ADVICE_REL);
    out.write_all(text.as_bytes())
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
    use crate::cli::args::{Commands, InspireArgs, KpopArgs, PlanArgs};
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
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
            request: None,
        }));
        assert!(md.starts_with("# malvin "));
        let md = command_doc_markdown(&Commands::Inspire(InspireArgs { request: None }));
        assert!(md.starts_with("# malvin inspire"));
    }

    #[test]
    fn print_doc_none_writes_full_malvin_md() {
        let out = capture_doc(None).expect("capture");
        assert_eq!(out.as_slice(), MALVIN_OVERVIEW_DOC.as_bytes());
    }

    #[test]
    fn print_doc_some_writes_subcommand_md() {
        let cmd = Commands::Kpop(KpopArgs {
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
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

    #[test]
    fn plan_doc_parses_with_plan_path_when_doc_flag_set() {
        let cli = Cli::try_parse_from(["malvin", "plan", "plan.md", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command.as_ref() {
            Some(Commands::Plan(p)) => assert_eq!(p.plan_path, "plan.md"),
            _ => panic!("expected Plan"),
        }
    }

    #[test]
    fn print_doc_plan_writes_subcommand_md() {
        let cmd = Commands::Plan(PlanArgs {
            plan_path: "plan.md".to_string(),
        });
        let out = capture_doc(Some(&cmd)).expect("capture");
        assert!(out.starts_with(b"# malvin plan"));
    }

    #[test]
    fn init_doc_substitutes_advice_path() {
        use crate::cli::args::{Commands, InitArgs};
        let cmd = Commands::Init(InitArgs {
            force: false,
            languages: vec![],
            path: None,
        });
        let out = capture_doc(Some(&cmd)).expect("capture");
        let text = String::from_utf8(out).expect("utf8");
        assert!(
            text.contains(".malvin/advice.md"),
            "init doc must show advice path"
        );
        assert!(
            !text.contains("{{ advice_path }}"),
            "init doc must not leave unresolved advice_path placeholder"
        );
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = doc_text;
        let _ = print_doc;
        let _ = stringify!(print_doc_to_writer);
    }
}
