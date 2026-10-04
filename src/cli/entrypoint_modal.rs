use std::ffi::OsString;
use std::path::{Path, PathBuf};

use malvin::modal_run::options::ModalOptions;
use malvin::modal_run::{ModalInvocation, reject_unsupported, remote_args, run_modal};

use super::{Exit, print_command_error};
use crate::cli::args::Cli;

pub(crate) fn modal_invocation(
    cli: &Cli,
    raw: &[OsString],
    options: ModalOptions,
) -> Result<ModalInvocation, String> {
    let set_flags: Vec<&str> = [("--watch", cli.router.watch), ("--iml", cli.shared.iml)]
        .into_iter()
        .filter_map(|(flag, on)| on.then_some(flag))
        .collect();
    reject_unsupported(&set_flags)?;
    if cli.command.is_some() {
        return Err("`--modal` cannot be combined with a subcommand".to_string());
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
    match modal_invocation(cli, raw, options).and_then(|inv| run_modal(&inv)) {
        Ok(0) => Exit::Success,
        Ok(_) => Exit::Failure,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}
