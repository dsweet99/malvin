//! CLI argument structs for the `malvin` binary.

use clap::{Args, Parser, Subcommand};

use super::do_flow::DoArgs;
use super::init_cmd::InitArgs;
use super::models_cmd::ModelsArgs;
use super::shared_opts::SharedOpts;

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
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Templates, kiss, pre-commit, Git LFS.
    Init(InitArgs),
    /// Send a prompt to the agent.
    Do(DoArgs),
    /// Implement → review → learn.
    Code(CodeArgs),
    /// KPOP investigation loop.
    Kpop(KpopArgs),
    /// List available models.
    Models(ModelsArgs),
}

#[derive(Args, Debug)]
pub struct CodeArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Implement → review → learn loop budget.
    #[arg(long, default_value_t = 5)]
    pub max_loops: usize,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Skip plan validation step.
    #[arg(long, default_value_t = false)]
    pub trust_the_plan: bool,
    /// Request or `@file` → `_malvin/.../plan.md`.
    pub request: String,
}

#[derive(Args, Debug)]
pub struct KpopArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Total KPOP + MBC2 hypothesis steps (## Step headings in the exp log) before stopping.
    #[arg(long, default_value_t = 10)]
    pub max_hypotheses: usize,
    /// Drives mean KPOP block size and MBC2 interleave; higher = more frequent MBC2 turns and smaller KPOP blocks. Non-finite or ≤ 0 disables MBC2 turns (pure multiturn KPOP).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../request.md`.
    pub request: String,
}
