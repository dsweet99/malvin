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

pub(crate) fn remote_do_dm_opts(cli: &Cli, interactive: bool) -> malvin::output::DoDmStdoutOpts {
    let dm_only = cli.has_do_request() && !cli.shared.verbose;
    malvin::output::DoDmStdoutOpts {
        enabled: dm_only,
        emit_markdown: dm_only && interactive && cli.shared.acp_stdout_markdown_enabled(),
    }
}

pub(crate) fn run_modal_route(cli: &Cli, raw: &[OsString], options: ModalOptions) -> Exit {
    let interactive = malvin::output::agent_stdout_tee_enabled();
    malvin::output::set_do_dm_stdout_opts(remote_do_dm_opts(cli, interactive));
    let result = modal_invocation(cli, raw, options).and_then(|inv| run_modal(&inv));
    malvin::output::set_do_dm_stdout_opts(malvin::output::DoDmStdoutOpts::default());
    match result {
        Ok(0) => Exit::Success,
        Ok(_) => Exit::Failure,
        Err(e) => {
            print_command_error(&e);
            Exit::Failure
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::remote_do_dm_opts;
    use crate::cli::Cli;

    fn opts(args: &[&str], interactive: bool) -> (bool, bool) {
        let cli = Cli::try_parse_from(args).expect("parse");
        let o = remote_do_dm_opts(&cli, interactive);
        (o.enabled, o.emit_markdown)
    }

    #[test]
    fn remote_do_is_dm_only_unless_verbose() {
        assert_eq!(opts(&["malvin", "--do", "hi"], true), (true, true));
        assert_eq!(opts(&["malvin", "--do", "hi"], false), (true, false));
        assert_eq!(opts(&["malvin", "-v", "--do", "hi"], true), (false, false));
        assert_eq!(opts(&["malvin", "hi"], true), (false, false));
    }
}
