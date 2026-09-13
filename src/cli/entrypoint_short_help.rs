use super::Exit;
use crate::cli::args::Cli;

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
    None
}
