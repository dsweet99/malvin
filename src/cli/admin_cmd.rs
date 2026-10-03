use clap::{Args, Subcommand};

use super::models_cmd::ModelsArgs;

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin admin <COMMAND>")]
pub struct AdminArgs {
    /// `None` only with `--doc`; parsing rejects a bare `malvin admin` otherwise.
    #[command(subcommand)]
    pub command: Option<AdminCommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommand {
    /// List available models
    Models(ModelsArgs),
    /// Reset herdr agent state to idle (not working)
    #[command(name = "reset-herdr", visible_alias = "rh")]
    ResetHerdr,
    /// Install the Cursor SDK (needs Node.js >= 22.13 and npm) for cursor: models
    #[command(name = "setup-cursor")]
    SetupCursor,
}

pub(crate) const ADMIN_MISSING_SUBCOMMAND: &str = "'malvin admin' requires a subcommand but one was not provided\n  [subcommands: models, reset-herdr, rh, setup-cursor]\n\nUsage: malvin admin <COMMAND>\n\nFor more information, try 'malvin admin --help' or 'malvin admin --doc'.";

pub fn run_admin(args: AdminArgs, current_model: &str) -> Result<(), String> {
    let Some(command) = args.command else {
        return Err(ADMIN_MISSING_SUBCOMMAND.to_string());
    };
    match command {
        AdminCommand::Models(models) => super::models_cmd::run_models(models, current_model),
        AdminCommand::ResetHerdr => {
            malvin::herdr::reset_to_not_working()?;
            malvin::output::print_stdout_line(
                malvin::output::MALVIN_WHO,
                "herdr state reset to idle (not working)",
            );
            Ok(())
        }
        AdminCommand::SetupCursor => run_setup_cursor(),
    }
}

fn run_setup_cursor() -> Result<(), String> {
    use malvin::cursor_sdk::bridge_install;
    let dest = bridge_install::default_install_dir();
    bridge_install::install_into(&dest).map_err(|e| bridge_install::install_failed_message(&e))?;
    malvin::output::print_stdout_line(
        malvin::output::MALVIN_WHO,
        &format!("Cursor SDK bridge ready in {}", dest.display()),
    );
    Ok(())
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
                command: Some(AdminCommand::ResetHerdr),
            })) => {}
            other => panic!("expected Admin::ResetHerdr, got {other:?}"),
        }
    }

    #[test]
    fn parses_admin_rh_alias_for_reset_herdr() {
        let cli = Cli::try_parse_from(["malvin", "admin", "rh"]).expect("parse");
        match cli.command {
            Some(Commands::Admin(AdminArgs {
                command: Some(AdminCommand::ResetHerdr),
            })) => {}
            other => panic!("expected Admin::ResetHerdr via rh, got {other:?}"),
        }
    }

    #[test]
    fn parses_admin_setup_cursor() {
        let cli = Cli::try_parse_from(["malvin", "admin", "setup-cursor"]).expect("parse");
        assert!(matches!(
            cli.command,
            Some(Commands::Admin(AdminArgs {
                command: Some(AdminCommand::SetupCursor),
            }))
        ));
    }

    #[test]
    fn parses_admin_models() {
        let cli =
            Cli::try_parse_from(["malvin", "admin", "models", "--refresh", "rpi:"]).expect("parse");
        match cli.command {
            Some(Commands::Admin(AdminArgs {
                command: Some(AdminCommand::Models(args)),
            })) => {
                assert!(args.refresh);
                assert_eq!(args.words, vec!["rpi:".to_string()]);
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
        assert!(help.contains("--doc"), "admin keeps --doc; got:\n{help}");
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
