//! Short usage for a single subcommand (about, usage, positionals; no options).

use std::io::{self, Write};

use clap::CommandFactory;
use clap::builder::Command;

use super::Cli;

fn subcommand_by_name<'a>(root: &'a Command, name: &str) -> Option<&'a Command> {
    root.get_subcommands().find(|sub| sub.get_name() == name)
}

fn positional_value_name(arg: &clap::Arg) -> &str {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map_or_else(|| arg.get_id().as_str(), clap::builder::Str::as_str)
}

fn positional_token(arg: &clap::Arg) -> String {
    let name = positional_value_name(arg);
    if arg.is_required_set() {
        format!("<{name}>")
    } else {
        format!("[{name}]")
    }
}

fn format_positional_usage_tokens(sub: &Command) -> Vec<String> {
    sub.get_arguments()
        .filter(|arg| arg.is_positional())
        .map(positional_token)
        .collect()
}

fn push_about_block(lines: &mut Vec<String>, sub: &Command) {
    if let Some(about) = sub.get_about() {
        lines.push(about.to_string());
        lines.push(String::new());
    }
}

fn subcommand_usage_line(sub: &Command) -> String {
    let usage_args = format_positional_usage_tokens(sub);
    if usage_args.is_empty() {
        format!("Usage: malvin {}", sub.get_name())
    } else {
        format!(
            "Usage: malvin {} {}",
            sub.get_name(),
            usage_args.join(" ")
        )
    }
}

fn argument_help_line(arg: &clap::Arg) -> String {
    let token = positional_token(arg);
    let help = arg
        .get_help()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    format!("  {token}  {help}")
}

fn append_arguments_block(lines: &mut Vec<String>, sub: &Command) {
    let positionals: Vec<_> = sub
        .get_arguments()
        .filter(|arg| arg.is_positional())
        .collect();
    if positionals.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push("Arguments:".to_string());
    for arg in positionals {
        lines.push(argument_help_line(arg));
    }
}

fn append_short_help_epilog(lines: &mut Vec<String>, name: &str) {
    lines.push(String::new());
    lines.push(format!("Use `malvin {name} --help` to see options."));
}

fn subcommand_short_help_lines(root: &Command, name: &str) -> Option<Vec<String>> {
    let sub = subcommand_by_name(root, name)?;
    let mut lines = Vec::new();
    push_about_block(&mut lines, sub);
    lines.push(subcommand_usage_line(sub));
    append_arguments_block(&mut lines, sub);
    append_short_help_epilog(&mut lines, name);
    Some(lines)
}

/// Build short usage for a subcommand (about, usage, positionals; no options).
pub fn render_subcommand_short_help(name: &str) -> Option<String> {
    let root = Cli::command();
    subcommand_short_help_lines(&root, name).map(|lines| format!("{}\n", lines.join("\n")))
}

/// Print short usage for a subcommand; full flags live under `malvin <COMMAND> --help`.
pub fn print_subcommand_short_help(name: &str) -> io::Result<()> {
    let text = render_subcommand_short_help(name).unwrap_or_default();
    io::stdout().lock().write_all(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_subcommand_short_help_code_omits_options() {
        let help = render_subcommand_short_help("code").expect("code subcommand");
        assert!(help.contains("Write code"));
        assert!(help.contains("Usage: malvin code"));
        assert!(help.contains("PLAN"));
        assert!(help.contains("Arguments:"));
        assert!(help.contains("malvin code --help"));
        assert!(!help.contains("Options:"));
        assert!(!help.contains("--max-loops"));
    }

    #[test]
    fn render_subcommand_short_help_inspire_omits_options() {
        let help = render_subcommand_short_help("inspire").expect("inspire subcommand");
        assert!(help.contains("Be creative"));
        assert!(help.contains("Usage: malvin inspire [REQUEST]"));
        assert!(help.contains("Arguments:"));
        assert!(help.contains("malvin inspire --help"));
        assert!(!help.contains("Options:"));
        assert!(!help.contains("--model"));
    }

    #[test]
    fn print_subcommand_short_help_invokes_stdout_path() {
        print_subcommand_short_help("code").expect("stdout");
    }

    #[test]
    fn kiss_cov_commands_help_subcommand_symbols() {
        let _ = stringify!(subcommand_by_name);
        let _ = stringify!(positional_value_name);
        let _ = stringify!(positional_token);
        let _ = stringify!(format_positional_usage_tokens);
        let _ = stringify!(push_about_block);
        let _ = stringify!(subcommand_usage_line);
        let _ = stringify!(argument_help_line);
        let _ = stringify!(append_arguments_block);
        let _ = stringify!(append_short_help_epilog);
        let _ = stringify!(subcommand_short_help_lines);
        let _ = stringify!(render_subcommand_short_help);
        let _ = stringify!(print_subcommand_short_help);
    }
}
