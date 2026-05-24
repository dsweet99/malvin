//! CLI argument structs for the `malvin` binary.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::tidy_flow::TidyArgs;

pub use super::models_cmd::ModelsArgs;
pub use crate::do_flow::DoArgs;
pub use crate::ideas_flow::IdeasArgs as InventArgs;
pub use crate::init_cmd::InitArgs;

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

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prep this repo
    Init(InitArgs),
    /// Respond to a single, generic request
    Do(DoArgs),
    /// Be creative
    #[command(name = "invent")]
    Invent(InventArgs),
    /// Write code
    Code(CodeArgs),
    /// Reason scientifically (q&a, research, optimization, ...)
    Kpop(KpopArgs),
    /// `KPop` bug hunter: find & fix bugs
    #[command(name = "hunt")]
    Hunt(BugArgs),
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
#[allow(clippy::struct_excessive_bools)]
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
    /// Alias for `--skip-pre-checks --trust-the-plan`.
    #[arg(short = 'f', default_value_t = false)]
    pub fast: bool,
    /// Request text or path to an existing `.md` file → `.malvin/logs/.../plan.md`.
    pub request: Option<String>,
}
