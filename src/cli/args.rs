//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::tidy_flow::TidyArgs;

pub use super::models_cmd::ModelsArgs;
pub use crate::do_flow::DoArgs;
pub use crate::ideas_flow::IdeasArgs as InventArgs;
pub use crate::init_cmd::InitArgs;

pub use super::args_bug_kpop::KpopArgs;
pub use super::shared_opts::GlobalOpts;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive CLI agent, via Cursor ACP",
    disable_help_subcommand = true,
    after_help = "Bare invocation: malvin REQUEST runs kpop; malvin --do REQUEST runs do; malvin @code|@constrain|@tidy … runs that workflow."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(flatten)]
    pub shared: SharedOpts,
    /// One-shot agent request (replaces `malvin do REQUEST`).
    #[arg(long = "do")]
    pub do_mode: bool,
    /// With `--do`: run repository quality gates before the prompt.
    #[arg(long = "repo-gates", default_value_t = false)]
    pub do_repo_gates: bool,
    /// With `--do`: show agent thought tokens on stdout when interactive.
    #[arg(long = "thoughts", default_value_t = false)]
    pub do_thoughts: bool,
    /// Gate-loop iterations for bare `malvin REQUEST` / `@workflow` invocations.
    #[arg(long = "max-loops", default_value_t = 1)]
    pub bare_max_loops: usize,
    /// `KPop` hypothesis budget per gate session for bare invocations.
    #[arg(long = "max-hypotheses", default_value_t = 10)]
    pub bare_max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999` for bare invocations.
    #[arg(long = "tenacious", default_value_t = false)]
    pub bare_tenacious: bool,
    /// When no subcommand: kpop request, `--do` request, or `@workflow` selector plus optional request.
    #[arg(value_name = "ARG")]
    pub bare_args: Vec<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prep this repo
    Init(InitArgs),
    /// Respond to a single, generic request (prefer `malvin --do REQUEST`)
    #[command(hide = true)]
    Do(DoArgs),
    /// Be creative
    #[command(name = "invent")]
    Invent(InventArgs),
    /// Write code
    Code(crate::cli::code_flow::CodeArgs),
    /// Write a regression test and code to satisfy constraints
    Constrain(crate::cli::constrain_flow::ConstrainArgs),
    /// Reason scientifically (prefer bare `malvin REQUEST`)
    #[command(hide = true)]
    Kpop(KpopArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// List available models
    Models(ModelsArgs),
}
