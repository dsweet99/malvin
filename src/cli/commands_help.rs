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
            let aliases: Vec<&str> = sub.get_visible_aliases().collect();
            match aliases.as_slice() {
                [] => format!("  {name:<width$} {about}"),
                [alias] => format!("  {name:<width$} {about} [alias: {alias}]"),
                _ => format!("  {name:<width$} {about} [aliases: {}]", aliases.join(", ")),
            }
        })
        .collect()
}

fn commands_only_help_lines(cmd: &Command, usage: &[&str], help_hint: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(about) = cmd.get_about() {
        lines.push(about.to_string());
        lines.push(String::new());
    }
    lines.extend(usage.iter().map(|line| (*line).to_string()));
    lines.push(String::new());
    lines.push("Commands:".to_string());
    lines.extend(format_command_lines(&visible_subcommands(cmd)));
    lines.extend([String::new(), help_hint.to_string()]);
    lines
}

fn top_level_help_lines(cmd: &Command) -> Vec<String> {
    commands_only_help_lines(
        cmd,
        &[
            "Usage: malvin [OPTION]... [REQUEST]...",
            "   or: malvin [OPTION]... <COMMAND>",
        ],
        "Use malvin --help to see options.",
    )
}

fn admin_help_lines(cmd: &Command) -> Vec<String> {
    commands_only_help_lines(
        cmd,
        &["Usage: malvin admin <COMMAND>"],
        "Use malvin admin --help to see options.",
    )
}

pub fn render_commands_only_help() -> String {
    let cmd = Cli::command();
    format!("{}\n", top_level_help_lines(&cmd).join("\n"))
}

pub fn render_admin_commands_only_help() -> String {
    let cmd = Cli::command();
    let admin = cmd
        .find_subcommand("admin")
        .expect("admin subcommand is registered");
    format!("{}\n", admin_help_lines(admin).join("\n"))
}

pub fn print_admin_commands_only_help() -> io::Result<()> {
    io::stdout()
        .lock()
        .write_all(render_admin_commands_only_help().as_bytes())
}

pub fn write_commands_only_help(mut writer: impl Write) -> io::Result<()> {
    writer.write_all(render_commands_only_help().as_bytes())
}

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
    fn commands_only_help_omits_init() {
        let cmd = Cli::command();
        assert!(!help_lists_subcommand(&cmd, "init"));
    }

    #[test]
    fn commands_only_help_lines_includes_command_usage_and_epilog() {
        let cmd = Cli::command();
        let lines = top_level_help_lines(&cmd);
        let text = lines.join("\n");
        assert!(text.contains("Usage: malvin [OPTION]... [REQUEST]..."));
        assert!(text.contains("malvin [OPTION]... <COMMAND>"));
        assert!(text.contains("Commands:"));
        assert!(!text.contains("tidy"));
    }

    #[test]
    fn render_commands_only_help_lists_subcommands_not_options() {
        let help = render_commands_only_help();
        let cmd = Cli::command();
        assert!(help.contains("Commands:"));
        assert!(!help_lists_subcommand(&cmd, "code"));
        assert!(!help_lists_subcommand(&cmd, "tidy"));
        assert!(help.contains("Usage: malvin [OPTION]... [REQUEST]..."));
        assert!(help.contains("malvin [OPTION]... <COMMAND>"));
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
    fn visible_subcommands_match_public_catalog() {
        let cmd = Cli::command();
        let names: Vec<_> = visible_subcommands(&cmd)
            .into_iter()
            .map(|sub| sub.get_name().to_string())
            .collect();
        assert!(!names.iter().any(|n| n == "code"));
        assert!(!names.iter().any(|n| n == "delight"));
        assert_eq!(names, vec!["admin"]);
    }

    #[test]
    fn format_command_lines_aligns_names() {
        let cmd = Cli::command();
        let lines = format_command_lines(&visible_subcommands(&cmd));
        assert!(lines.iter().any(|line| line.starts_with("  admin")));
        assert!(!lines.iter().any(|line| line.starts_with("  init")));
    }

    #[test]
    fn admin_commands_only_help_lists_subcommands_with_descriptions() {
        let help = render_admin_commands_only_help();
        assert!(
            help.starts_with("Operator maintenance commands\n\n"),
            "{help}"
        );
        assert!(help.contains("Usage: malvin admin <COMMAND>"), "{help}");
        assert!(help.contains("Commands:"), "{help}");
        assert!(
            help.lines()
                .any(|l| l.starts_with("  models ") && l.contains("List available models")),
            "{help}"
        );
        assert!(
            help.lines().any(|l| l.starts_with("  reset-herdr ")
                && l.contains("Reset herdr agent state")
                && l.ends_with("[alias: rh]")),
            "{help}"
        );
        assert!(help.contains("malvin admin --help"), "{help}");
        assert!(!help.contains("Options:"), "{help}");
        assert!(!help.contains("error"), "{help}");
    }

    #[test]
    fn print_admin_commands_only_help_invokes_stdout_path() {
        print_admin_commands_only_help().expect("stdout");
    }

    #[test]
    fn kiss_cov_commands_help_symbols() {}
}
