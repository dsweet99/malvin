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
    about = "Implement / review / learn via Cursor agent ACP",
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
    /// Single ACP coder prompt.
    Do(DoArgs),
    /// Implement → review → learn.
    Code(CodeArgs),
    /// KPOP hypothesis loop.
    Kpop(KpopArgs),
    /// Models from `agent` / `cursor-agent`.
    Models(ModelsArgs),
}

#[derive(Args, Debug)]
pub struct CodeArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Implement → review → learn loop budget.
    #[arg(long, default_value_t = 5)]
    pub max_loops: usize,
    /// Skip learn (memory update).
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../plan.md`.
    pub request: String,
}

#[derive(Args, Debug)]
pub struct KpopArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// KPOP loop budget.
    #[arg(long, default_value_t = 10)]
    pub max_loops: usize,
    /// P(mbc2 creative) after the first 3 prompts (0–1).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learn (memory update).
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../request.md`.
    pub request: String,
}
