use super::{Exit, print_command_error};
use crate::cli::args::Cli;

fn entrypoint_doc_exit(cli: &Cli) -> Exit {
    match crate::cli::command_docs::print_doc_for_cli(cli) {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_advice_exit(tag: &str) -> Exit {
    match crate::cli::advice_cmd::print_advice(tag) {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_advice_list_exit() -> Exit {
    match crate::cli::advice_cmd::print_advice_list() {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

pub(crate) fn entrypoint_advice_or_doc_exit(cli: &Cli) -> Option<Exit> {
    if let Some(tag) = cli.shared.advice.as_deref() {
        if tag.is_empty() {
            return Some(entrypoint_advice_list_exit());
        }
        return Some(entrypoint_advice_exit(tag));
    }
    if cli.shared.doc {
        return Some(entrypoint_doc_exit(cli));
    }
    None
}
