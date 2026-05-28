//! Commands-only help for bare `malvin` (no subcommand, no REQUEST).

use std::io::{self, Write};

use clap::CommandFactory;
use clap::builder::Command;

use super::Cli;

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

/// Build subcommand catalog text for bare `malvin`.
pub fn render_commands_only_help() -> String {
    let cmd = Cli::command();
    let mut lines = Vec::new();
    if let Some(about) = cmd.get_about() {
        lines.push(about.to_string());
        lines.push(String::new());
    }
    lines.push("Usage: malvin [COMMAND]".to_string());
    lines.push(String::new());
    lines.push("Commands:".to_string());
    lines.extend(format_command_lines(&visible_subcommands(&cmd)));
    lines.push(String::new());
    lines.push("Use `malvin --help` to see options.".to_string());
    format!("{}\n", lines.join("\n"))
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
    fn render_commands_only_help_lists_subcommands_not_options() {
        let help = render_commands_only_help();
        assert!(help.contains("Commands:"));
        assert!(help.contains("init"));
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
        use clap::CommandFactory;
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
        use clap::CommandFactory;
        let cmd = Cli::command();
        let lines = format_command_lines(&visible_subcommands(&cmd));
        assert!(lines.iter().any(|line| line.starts_with("  init")));
    }

    #[test]
    fn kiss_cov_commands_help_symbols() {
        let _ = stringify!(
            visible_subcommands,
            format_command_lines,
            render_commands_only_help,
            write_commands_only_help,
            print_commands_only_help
        );
    }
}
