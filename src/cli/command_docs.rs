
use std::io::{self, Write};

use super::Commands;
use crate::cli::args::Cli;

pub(crate) const MALVIN_OVERVIEW_DOC: &str = include_str!("../../default_prompts/docs/malvin.md");

pub(crate) const ROUTER_DOC: &str = include_str!("../../default_prompts/docs/router.md");

pub(crate) const DO_DOC: &str = include_str!("../../default_prompts/docs/do.md");

const fn gate_loop_command_doc(cmd: &Commands) -> Option<&'static str> {
    match cmd {
        Commands::Tidy(_) => Some(include_str!("../../default_prompts/docs/tidy.md")),
        Commands::Write(_) => Some(include_str!("../../default_prompts/docs/write.md")),
        _ => None,
    }
}

pub(crate) const fn command_doc_markdown(cmd: &Commands) -> &'static str {
    if let Some(doc) = gate_loop_command_doc(cmd) {
        return doc;
    }
    match cmd {
        Commands::Inspire(_) | Commands::Adaptix(_) => {
            include_str!("../../default_prompts/docs/inspire.md")
        }
        Commands::Models(_) => include_str!("../../default_prompts/docs/models.md"),
        Commands::Init(_) => include_str!("../../default_prompts/docs/init.md"),
        _ => panic!("uncovered command doc"),
    }
}

#[must_use]
pub(crate) fn doc_text(command: Option<&Commands>) -> String {
    command.map_or_else(
        || format!("{MALVIN_OVERVIEW_DOC}\n---\n\n{ROUTER_DOC}"),
        |cmd| command_doc_markdown(cmd).to_string(),
    )
}

pub(crate) fn print_doc_to_writer(command: Option<&Commands>, mut out: impl Write) -> Result<(), String> {
    let text = doc_text(command).replace("{{ advice_path }}", crate::MALVIN_ADVICE_REL);
    out.write_all(text.as_bytes())
        .map_err(|e| format!("stdout: {e}"))?;
    Ok(())
}

pub(crate) fn print_doc_for_cli_to_writer(cli: &Cli, out: impl Write) -> Result<(), String> {
    if cli.do_workflow {
        let mut out = out;
        let text = DO_DOC.replace("{{ advice_path }}", crate::MALVIN_ADVICE_REL);
        return out
            .write_all(text.as_bytes())
            .map_err(|e| format!("stdout: {e}"));
    }
    print_doc_to_writer(cli.command.as_ref(), out)
}

pub(crate) fn print_doc_for_cli(cli: &Cli) -> Result<(), String> {
    print_doc_for_cli_to_writer(cli, io::stdout().lock())
}

#[cfg(test)]
#[path = "command_docs_tests.rs"]
mod command_docs_tests;
