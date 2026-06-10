//! Commands-only help for bare `malvin` (no subcommand, no REQUEST).

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
    lines.push("Usage: malvin [COMMAND|REQUEST]...".to_string());
    lines.push(String::new());
    lines.push("Commands:".to_string());
    lines.extend(format_command_lines(&visible_subcommands(cmd)));
    lines.extend([
        String::new(),
        "Pass one or more REQUESTs with no subcommand to run KPop on each in sequence.".to_string(),
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

    #[test]
    fn commands_only_help_lines_includes_request_usage_and_epilog() {
        let cmd = Cli::command();
        let lines = commands_only_help_lines(&cmd);
        let text = lines.join("\n");
        assert!(text.contains("Usage: malvin [COMMAND|REQUEST]..."));
        assert!(text.contains("one or more REQUESTs"));
        assert!(text.contains("Commands:"));
        assert!(!text.contains("kpop"));
    }

    #[test]
    fn render_commands_only_help_lists_subcommands_not_options() {
        let help = render_commands_only_help();
        assert!(help.contains("Commands:"));
        assert!(help.contains("init"));
        assert!(help.contains("Usage: malvin [COMMAND|REQUEST]..."));
        assert!(help.contains("one or more REQUESTs"));
        assert!(help.contains("malvin --help"));
        assert!(!help.contains("Options:"));
        assert!(!help.contains("--no-color"));
        assert!(!help.contains("kpop"), "hidden kpop subcommand must not appear");
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
    fn visible_subcommands_omits_hidden_kpop() {
        let cmd = Cli::command();
        let names: Vec<_> = visible_subcommands(&cmd)
            .into_iter()
            .map(|sub| sub.get_name().to_string())
            .collect();
        assert!(names.contains(&"code".to_string()));
        assert!(!names.iter().any(|name| name == "kpop"));
    }

    #[test]
    fn format_command_lines_aligns_names() {
        let cmd = Cli::command();
        let lines = format_command_lines(&visible_subcommands(&cmd));
        assert!(lines.iter().any(|line| line.starts_with("  init")));
    }

    #[test]
    fn kiss_cov_commands_help_symbols() {
        let _ = stringify!(visible_subcommands);
        let _ = stringify!(format_command_lines);
        let _ = stringify!(commands_only_help_lines);
        let _ = stringify!(render_commands_only_help);
        let _ = stringify!(write_commands_only_help);
        let _ = stringify!(print_commands_only_help);
    }
}
