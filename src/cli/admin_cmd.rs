use clap::{Args, Subcommand};

use super::models_cmd::ModelsArgs;

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin admin <COMMAND>")]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommand {
    /// List available models
    Models(ModelsArgs),
    /// Reset herdr agent state to idle (not working)
    #[command(name = "reset-herdr")]
    ResetHerdr,
}

pub fn run_admin(args: AdminArgs, current_model: &str) -> Result<(), String> {
    match args.command {
        AdminCommand::Models(models) => super::models_cmd::run_models(models, current_model),
        AdminCommand::ResetHerdr => {
            crate::herdr::reset_to_not_working()?;
            crate::output::print_stdout_line(
                crate::output::MALVIN_WHO,
                "herdr state reset to idle (not working)",
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AdminArgs, AdminCommand};
    use crate::cli::{Cli, Commands};
    use clap::Parser;

    #[test]
    fn parses_admin_reset_herdr() {
        let cli = Cli::try_parse_from(["malvin", "admin", "reset-herdr"]).expect("parse");
        match cli.command {
            Some(Commands::Admin(AdminArgs {
                command: AdminCommand::ResetHerdr,
            })) => {}
            other => panic!("expected Admin::ResetHerdr, got {other:?}"),
        }
    }

    #[test]
    fn parses_admin_models() {
        let cli = Cli::try_parse_from(["malvin", "admin", "models", "--refresh", "pi:"]).expect("parse");
        match cli.command {
            Some(Commands::Admin(AdminArgs {
                command: AdminCommand::Models(args),
            })) => {
                assert!(args.refresh);
                assert_eq!(args.words, vec!["pi:".to_string()]);
            }
            other => panic!("expected Admin::Models, got {other:?}"),
        }
    }

    #[test]
    fn admin_help_omits_agent_session_flags() {
        use clap::CommandFactory;
        let help = Cli::command()
            .try_get_matches_from(["malvin", "admin", "--help"])
            .expect_err("help")
            .to_string();
        for needle in [
            "--model",
            "--no-force",
            "--no-tenacious",
            "--gates",
            "-g,",
            "--quiet",
            "--verbose",
            "--max-acp-retries",
            "--name",
            "--git",
            "--creative",
            "-b,",
            "--background",
        ] {
            assert!(
                !help.contains(needle),
                "admin help must not present {needle}; got:\n{help}"
            );
        }
        assert!(
            help.contains("--doc"),
            "admin keeps --doc; got:\n{help}"
        );
    }

    #[test]
    fn admin_models_help_omits_agent_session_flags() {
        use clap::CommandFactory;
        let help = Cli::command()
            .try_get_matches_from(["malvin", "admin", "models", "--help"])
            .expect_err("help")
            .to_string();
        for needle in [
            "--model",
            "--no-force",
            "--no-tenacious",
            "--gates",
            "-g,",
            "--quiet",
            "--verbose",
            "--max-acp-retries",
            "--name",
            "--git",
            "--creative",
            "-b,",
            "--background",
        ] {
            assert!(
                !help.contains(needle),
                "admin models help must not present {needle}; got:\n{help}"
            );
        }
        assert!(
            help.contains("--refresh"),
            "models keeps --refresh; got:\n{help}"
        );
    }
}
