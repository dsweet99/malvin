//! Built-in per-command documentation (`default_prompts/docs/*.md`), embedded at compile time.

use std::io::{self, Write};

use super::Commands;

const fn command_doc_markdown(cmd: &Commands) -> &'static str {
    match cmd {
        Commands::Init(_) => include_str!("../../default_prompts/docs/init.md"),
        Commands::Do(_) => include_str!("../../default_prompts/docs/do.md"),
        Commands::Code(_) => include_str!("../../default_prompts/docs/code.md"),
        Commands::Kpop(_) => include_str!("../../default_prompts/docs/kpop.md"),
        Commands::Bug(_) => include_str!("../../default_prompts/docs/bug.md"),
        Commands::Tidy(_) => include_str!("../../default_prompts/docs/tidy.md"),
        Commands::Plan(_) => include_str!("../../default_prompts/docs/plan.md"),
        Commands::Models(_) => include_str!("../../default_prompts/docs/models.md"),
    }
}

pub(crate) fn print_command_doc(cmd: &Commands) -> Result<(), String> {
    let text = command_doc_markdown(cmd);
    let mut out = io::stdout().lock();
    out.write_all(text.as_bytes())
        .map_err(|e| format!("stdout: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::args::{BugArgs, CodeArgs, Commands, KpopArgs, PlanArgs};
    use crate::cli::models_cmd::ModelsArgs;
    use crate::cli::tidy_flow::TidyArgs;
    use crate::cli::Cli;
    use crate::do_flow::DoArgs;
    use crate::init_cmd::InitArgs;
    use clap::Parser;

    #[test]
    fn doc_markdown_nonempty_for_each_subcommand() {
        let cases: [Commands; 8] = [
            Commands::Init(InitArgs {
                force: false,
                languages: vec![],
                path: None,
            }),
            Commands::Do(DoArgs {
                cooked: false,
                repo_gates: false,
                thoughts: false,
                request: None,
            }),
            Commands::Code(CodeArgs {
                max_loops: 1,
                no_learn: true,
                trust_the_plan: false,
                skip_pre_checks: false,
                request: None,
            }),
            Commands::Kpop(KpopArgs {
                max_hypotheses: 1,
                p_creative: 0.1,
                no_learn: true,
                request: None,
            }),
            Commands::Bug(BugArgs {
                max_hypotheses: 1,
                p_creative: 0.1,
                no_learn: true,
                skip_pre_checks: false,
            }),
            Commands::Tidy(TidyArgs {
                max_loops: 1,
                no_learn: true,
            }),
            Commands::Plan(PlanArgs {
                plan_path: None,
                text: None,
            }),
            Commands::Models(ModelsArgs {}),
        ];
        for cmd in cases {
            let md = command_doc_markdown(&cmd);
            assert!(!md.is_empty(), "doc body must not be empty");
            assert!(md.starts_with("# malvin "), "expected title line: {md:?}");
        }
    }

    #[test]
    fn kpop_doc_parses_without_request_when_doc_flag_set() {
        let cli = Cli::try_parse_from(["malvin", "kpop", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command {
            Commands::Kpop(k) => assert!(k.request.is_none()),
            _ => panic!("expected Kpop"),
        }
    }

    #[test]
    fn init_doc_parses_without_languages_when_doc_flag_set() {
        let cli = Cli::try_parse_from(["malvin", "init", "--doc"]).expect("parse");
        assert!(cli.shared.doc);
        match cli.command {
            Commands::Init(i) => assert!(i.languages.is_empty()),
            _ => panic!("expected Init"),
        }
    }

    #[test]
    fn print_command_doc_kpop_writes_markdown() {
        let cmd = Commands::Kpop(KpopArgs {
            max_hypotheses: 1,
            p_creative: 0.1,
            no_learn: true,
            request: None,
        });
        super::print_command_doc(&cmd).expect("stdout");
    }
}
