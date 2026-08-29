use super::{Commands, Exit};
use crate::cli::args::Cli;

pub(super) fn entrypoint_short_help_when_request_missing(
    doc: bool,
    request: Option<&String>,
    subcommand: &str,
) -> Option<Exit> {
    if doc || request.is_some() {
        return None;
    }
    let _ = crate::cli::commands_help::print_subcommand_short_help(subcommand);
    Some(Exit::Success)
}

pub(super) fn entrypoint_do_short_help() -> Exit {
    let text = "\
One-shot agent turn (non-looping)

Usage: malvin --do [OPTION]... [REQUEST]

Arguments:
  [REQUEST]  Existing `.md` path or literal text

Use malvin --help to see options.
";
    let _ = std::io::Write::write_all(&mut std::io::stdout().lock(), text.as_bytes());
    Exit::Success
}

pub(super) fn entrypoint_request_missing_short_help(cli: &Cli) -> Option<Exit> {
    if cli.do_workflow {
        if cli.shared.doc || cli.request.is_some() {
            return None;
        }
        return Some(entrypoint_do_short_help());
    }
    let command = cli.command.as_ref()?;
    let (request, subcommand) = match command {
        Commands::Inspire(inspire) | Commands::Adaptix(inspire) => {
            (inspire.request.as_ref(), "inspire")
        }
        Commands::Write(write_args) => (write_args.request.as_ref(), "write"),
        Commands::Models(_) | Commands::Admin(_) => return None,
    };
    entrypoint_short_help_when_request_missing(cli.shared.doc, request, subcommand)
}
