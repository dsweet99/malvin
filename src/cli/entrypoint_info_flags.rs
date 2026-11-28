use super::Exit;
use super::entrypoint_support::print_command_error;
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
    if tag.is_empty() {
        return entrypoint_advice_list_exit();
    }
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

fn entrypoint_facts_exit(tag: &str) -> Exit {
    let result = if tag.is_empty() {
        crate::cli::facts_view::print_facts_list()
    } else {
        crate::cli::facts_view::print_fact(tag)
    };
    match result {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

fn entrypoint_credits_exit() -> Exit {
    match crate::cli::credits_cmd::print_credits() {
        Ok(()) => Exit::Success,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

pub(crate) fn entrypoint_advice_or_doc_exit(cli: &Cli) -> Option<Exit> {
    if cli.shared.credits {
        return Some(entrypoint_credits_exit());
    }
    if let Some(tag) = cli.shared.facts.as_deref() {
        return Some(entrypoint_facts_exit(tag));
    }
    if let Some(tag) = cli.shared.advice.as_deref() {
        return Some(entrypoint_advice_exit(tag));
    }
    if cli.shared.doc {
        return Some(entrypoint_doc_exit(cli));
    }
    None
}
