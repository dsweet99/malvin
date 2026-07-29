//! Deprecated `malvin code` subcommand: detect argv and exit before substantive work.

use std::ffi::OsString;

use super::exit::Exit;
use crate::output::{MALVIN_WHO, print_stderr_line};

pub(crate) const CODE_DEPRECATED_MESSAGE: &str =
    "the `code` command is deprecated; use bare `malvin REQUEST` or `malvin inspire` instead";

pub(crate) fn exit_if_code_subcommand(args: &[OsString]) -> Option<Exit> {
    if argv_selects_code_subcommand(args) {
        print_stderr_line(MALVIN_WHO, CODE_DEPRECATED_MESSAGE);
        return Some(Exit::Failure);
    }
    None
}

fn argv_selects_code_subcommand(args: &[OsString]) -> bool {
    if args.len() < 2 {
        return false;
    }
    let tokens: Vec<String> = args
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let mut i = 1;
    while i < tokens.len() {
        let token = &tokens[i];
        if token == "--" {
            return false;
        }
        if token == "code" {
            return true;
        }
        if let Some(rest) = token.strip_prefix("--") {
            if rest.contains('=') {
                i += 1;
                continue;
            }
            if i + 1 < tokens.len() && !tokens[i + 1].starts_with('-') {
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if token.starts_with('-') {
            i += 1;
            continue;
        }
        return false;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(|s| OsString::from(*s)).collect()
    }

    #[test]
    fn detects_bare_code_subcommand() {
        assert!(argv_selects_code_subcommand(&os(&["malvin", "code"])));
    }

    #[test]
    fn detects_code_with_request() {
        assert!(argv_selects_code_subcommand(&os(&["malvin", "code", "plan.md"])));
    }

    #[test]
    fn detects_code_after_global_flags() {
        assert!(argv_selects_code_subcommand(&os(&[
            "malvin", "--model", "auto", "code", "plan.md"
        ])));
    }

    #[test]
    fn detects_code_help_and_doc() {
        assert!(argv_selects_code_subcommand(&os(&["malvin", "code", "--help"])));
        assert!(argv_selects_code_subcommand(&os(&["malvin", "code", "--doc"])));
    }

    #[test]
    fn ignores_other_subcommands() {
        assert!(!argv_selects_code_subcommand(&os(&["malvin", "kpop", "x"])));
        assert!(!argv_selects_code_subcommand(&os(&["malvin", "inspire"])));
    }

    #[test]
    fn ignores_bare_request() {
        assert!(!argv_selects_code_subcommand(&os(&["malvin", "plan.md"])));
    }

    #[test]
    fn ignores_top_level_help() {
        assert!(!argv_selects_code_subcommand(&os(&["malvin", "--help"])));
        assert!(!argv_selects_code_subcommand(&os(&["malvin", "--doc"])));
    }
}
