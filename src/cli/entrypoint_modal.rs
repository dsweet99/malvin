use std::ffi::OsString;
use std::path::{Path, PathBuf};

use malvin::modal_run::options::ModalOptions;
use malvin::modal_run::{ModalInvocation, reject_unsupported, remote_args, run_modal};

use super::Exit;
use super::entrypoint::print_command_error;
use crate::cli::args::Cli;

pub(crate) fn modal_invocation(
    cli: &Cli,
    raw: &[OsString],
    options: ModalOptions,
) -> Result<ModalInvocation, String> {
    let mut set_flags = Vec::new();
    if cli.shared.ml.is_forever() {
        set_flags.push("--ml=inf");
    }
    reject_unsupported(&set_flags)?;
    if cli.command.is_some() {
        return Err("`--remote` cannot be combined with a subcommand".to_string());
    }
    Ok(ModalInvocation {
        remote_args: remote_args(raw),
        model: cli.shared.model.canonical(),
        request_files: cli
            .requests
            .iter()
            .filter(|r| Path::new(r).is_file())
            .map(PathBuf::from)
            .collect(),
        options,
    })
}

pub(crate) fn run_modal_route(cli: &Cli, raw: &[OsString], options: ModalOptions) -> Exit {
    let result = modal_invocation(cli, raw, options).and_then(|inv| run_modal(&inv));
    match result {
        Ok(0) => Exit::Success,
        Ok(_) => Exit::Failure,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}
