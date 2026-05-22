//! CLI argument structs for the `malvin` binary.

use std::path::PathBuf;

use clap::{Args, CommandFactory, Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::tidy_flow::TidyArgs;

pub use crate::do_flow::DoArgs;
pub use crate::mbc2_flow::Mbc2Args;
pub use crate::init_cmd::InitArgs;
pub use super::models_cmd::ModelsArgs;

pub use super::args_bug_kpop::{BugArgs, KpopArgs};
pub use super::shared_opts::GlobalOpts;

#[derive(Parser, Debug)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive CLI agent, via Cursor ACP",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(flatten)]
    pub shared: SharedOpts,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

impl Cli {
    /// When `--doc` is absent, a subcommand is required (clap parse-time semantics).
    pub(crate) fn validate_subcommand(&self) -> Result<(), clap::Error> {
        if self.command.is_none() && !self.shared.doc {
            let mut cmd = Self::command();
            return Err(cmd.error(
                clap::error::ErrorKind::MissingSubcommand,
                "a subcommand is required",
            ));
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prep a workspace (repo)
    Init(InitArgs),
    /// Respond to a single request
    Do(DoArgs),
    /// MBC2 boundary exploration (one-shot ideation)
    Mbc2(Mbc2Args),
    /// Implement a plan
    Code(CodeArgs),
    /// Popperian scientific investigator
    Kpop(KpopArgs),
    /// KPOP bug hunter: find, regression test, fix (EXPERIMENTAL)
    Bug(BugArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// Write or review a plan file (EXPERIMENTAL)
    Plan(PlanArgs),
    /// List available models
    Models(ModelsArgs),
}

#[derive(Args, Debug)]
pub struct PlanArgs {
    #[arg(long = "plan_path", visible_alias = "plan-path", value_name = "PATH")]
    pub plan_path: Option<PathBuf>,
    #[arg(value_name = "TEXT")]
    pub text: Option<String>,
}

#[derive(Args, Debug)]
pub struct CodeArgs {
    /// Implement → review → learn loop budget.
    #[arg(long, default_value_t = 5)]
    pub max_loops: usize,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Skip plan validation step.
    #[arg(long, default_value_t = false)]
    pub trust_the_plan: bool,
    /// Skip workspace quality gates before the ACP session starts.
    #[arg(long, default_value_t = false)]
    pub skip_pre_checks: bool,
    /// Request or `@file` → `_malvin/.../plan.md`.
    pub request: Option<String>,
}

