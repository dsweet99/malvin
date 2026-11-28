use clap::{Args, Subcommand};

use super::facts_cmd::FactsArgs;
use super::models_cmd::ModelsArgs;
use super::remotes_cmd::RemotesArgs;

#[derive(Args, Debug, Clone)]
#[command(override_usage = "malvin admin <COMMAND>")]
pub struct AdminArgs {
    /// `None` for bare `malvin admin`, which prints the admin command catalog.
    #[command(subcommand)]
    pub command: Option<AdminCommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AdminCommand {
    /// List available models
    #[command(alias = "model")]
    Models(ModelsArgs),
    /// List remotes for `--remote` and their suboptions
    Remotes(RemotesArgs),
    /// Reset herdr agent state by releasing the pane
    #[command(name = "reset-herdr", visible_alias = "rh")]
    ResetHerdr,
    /// Add or remove facts in ~/.malvinconf/facts.jsonl
    #[command(override_usage = "malvin admin facts <add|remove> ...")]
    Facts(FactsArgs),
}
