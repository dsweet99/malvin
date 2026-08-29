use clap::{Args, Subcommand};

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin admin <COMMAND>")]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommand {
    /// Reset herdr agent state to idle (not working)
    #[command(name = "reset-herdr")]
    ResetHerdr,
}

pub fn run_admin(args: AdminArgs) -> Result<(), String> {
    match args.command {
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
}
