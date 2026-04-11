//! CLI argument structs for the `malvin` binary.

use clap::{Args, Parser, Subcommand};

use super::init_cmd::InitArgs;
use super::models_cmd::ModelsArgs;
use super::shared_opts::SharedOpts;

#[derive(Parser, Debug)]
#[command(
    name = "malvin",
    version,
    about = "Implementation and review workflow via agent acp",
    disable_help_subcommand = true
)]
#[allow(clippy::struct_excessive_bools)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run the full implement → review → learn workflow.
    Code(CodeArgs),
    /// KPOP hypothesis workflow.
    Kpop(KpopArgs),
    /// Initialize a repository with templates and local tooling (`kiss`, Git LFS, pre-commit).
    Init(InitArgs),
    /// List models from the Cursor agent CLI (`cursor-agent models` / `agent models`).
    Models(ModelsArgs),
}

#[derive(Args, Debug)]
pub struct CodeArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    #[arg(long, default_value_t = 5)]
    pub max_loops: usize,
    /// Do not learn (update memory).
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// `@path` reads a file; otherwise literal user text. Stored as `_malvin/.../plan.md`.
    pub request: String,
}

#[derive(Args, Debug)]
pub struct KpopArgs {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Hypothesis budget for the KPOP prompt.
    #[arg(long, default_value_t = 10)]
    pub max_loops: usize,
    /// Do not learn (update memory).
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// `@path` reads a file; otherwise literal user text. Stored as `_malvin/.../request.md`.
    pub request: String,
}
