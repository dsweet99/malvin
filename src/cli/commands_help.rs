//! Commands-only help for bare `malvin` (no subcommand).

use std::io::{self, Write};

use clap::CommandFactory;
use clap::builder::Command;

use super::Cli;

pub use super::commands_help_subcommand::print_subcommand_short_help;

fn visible_subcommands(cmd: &Command) -> Vec<&Command> {
    cmd.get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .collect()
}

fn format_command_lines(subs: &[&Command]) -> Vec<String> {
    let width = subs
        .iter()
        .map(|sub| sub.get_name().len())
        .max()
        .unwrap_or(0);
    subs.iter()
        .map(|sub| {
            let name = sub.get_name();
            let about = sub
                .get_about()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            format!("  {name:<width$} {about}")
        })
        .collect()
}

fn commands_only_help_lines(cmd: &Command) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(about) = cmd.get_about() {
        lines.push(about.to_string());
        lines.push(String::new());
    }
    lines.push("Usage: malvin [OPTIONS] [REQUEST]".to_string());
    lines.push("        malvin [OPTIONS] <COMMAND>".to_string());
    lines.push(String::new());
    lines.push("Commands:".to_string());
    lines.extend(format_command_lines(&visible_subcommands(cmd)));
    lines.extend([
        String::new(),
        "Use `malvin --help` to see options.".to_string(),
    ]);
    lines
}

/// Build subcommand catalog text for bare `malvin`.
pub fn render_commands_only_help() -> String {
    let cmd = Cli::command();
    format!("{}\n", commands_only_help_lines(&cmd).join("\n"))
}

/// Write subcommand catalog for bare `malvin`; full flags live under `malvin --help`.
pub fn write_commands_only_help(mut writer: impl Write) -> io::Result<()> {
    writer.write_all(render_commands_only_help().as_bytes())
}

/// Print subcommand catalog for bare `malvin`; full flags live under `malvin --help`.
pub fn print_commands_only_help() -> io::Result<()> {
    write_commands_only_help(io::stdout().lock())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn help_lists_subcommand(cmd: &Command, name: &str) -> bool {
        format_command_lines(&visible_subcommands(cmd))
            .iter()
            .any(|line| line.starts_with(&format!("  {name}")))
    }

    #[test]
    fn commands_only_help_lists_init() {
        let cmd = Cli::command();
        assert!(help_lists_subcommand(&cmd, "init"));
    }

    #[test]
    fn commands_only_help_lines_includes_command_usage_and_epilog() {
        let cmd = Cli::command();
        let lines = commands_only_help_lines(&cmd);
        let text = lines.join("\n");
        assert!(text.contains("Usage: malvin [OPTIONS] [REQUEST]"));
        assert!(text.contains("malvin [OPTIONS] <COMMAND>"));
        assert!(text.contains("Commands:"));
        assert!(text.contains("tidy"));
    }

    #[test]
    fn render_commands_only_help_lists_subcommands_not_options() {
        let help = render_commands_only_help();
        let cmd = Cli::command();
        assert!(help.contains("Commands:"));
        assert!(!help_lists_subcommand(&cmd, "code"));
        assert!(!help_lists_subcommand(&cmd, "kpop"));
        assert!(help_lists_subcommand(&cmd, "tidy"));
        assert!(help.contains("Usage: malvin [OPTIONS] [REQUEST]"));
        assert!(help.contains("malvin [OPTIONS] <COMMAND>"));
        assert!(help.contains("malvin --help"));
        assert!(!help.contains("Options:"));
        assert!(!help.contains("--no-color"));
    }

    #[test]
    fn write_commands_only_help_buffers_catalog() {
        let mut buf = Vec::new();
        write_commands_only_help(&mut buf).expect("write");
        let help = String::from_utf8(buf).expect("utf8");
        assert!(help.contains("Commands:"));
        assert!(!help.contains("Options:"));
    }

    #[test]
    fn print_commands_only_help_invokes_stdout_path() {
        print_commands_only_help().expect("stdout");
    }

    #[test]
    fn visible_subcommands_omits_kpop() {
        let cmd = Cli::command();
        let names: Vec<_> = visible_subcommands(&cmd)
            .into_iter()
            .map(|sub| sub.get_name().to_string())
            .collect();
        assert!(!names.iter().any(|n| n == "code"));
        assert!(!names.iter().any(|n| n == "kpop"));
        assert!(!names.iter().any(|n| n == "delight"));
        assert_eq!(
            names,
            vec!["init", "tidy", "write", "inspire", "models"]
        );
    }

    #[test]
    fn format_command_lines_aligns_names() {
        let cmd = Cli::command();
        let lines = format_command_lines(&visible_subcommands(&cmd));
        assert!(lines.iter().any(|line| line.starts_with("  tidy")));
        assert!(!lines.iter().any(|line| line.starts_with("  kpop")));
    }

    #[test]
    fn kiss_cov_commands_help_symbols() {
    }
}
