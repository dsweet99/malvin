//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::tidy_flow::TidyArgs;

pub use super::models_cmd::ModelsArgs;
pub use crate::do_flow::DoArgs;
pub use crate::ideas_flow::IdeasArgs as InventArgs;
pub use crate::init_cmd::InitArgs;
pub use crate::plan_flow::PlanArgs;

pub use super::args_bug_kpop::KpopArgs;
pub use super::shared_opts::GlobalOpts;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive CLI agent, via Cursor ACP",
    disable_help_subcommand = true,
    after_help = "Bare invocation: malvin REQUEST runs kpop (same as malvin kpop REQUEST). Use subcommands for do, code, and tidy."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Gate-loop iterations for bare `malvin REQUEST` (kpop).
    #[arg(long = "max-loops", default_value_t = 1)]
    pub bare_max_loops: usize,
    /// Number of hypotheses per `KPop` round for bare kpop invocations.
    #[arg(long = "max-hypotheses", default_value_t = 5)]
    pub bare_max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999` for bare kpop invocations.
    #[arg(long = "tenacious", default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub bare_tenacious: bool,
    /// When no subcommand: kpop request text or path.
    #[arg(value_name = "REQUEST")]
    pub bare_args: Vec<String>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Prep this repo
    Init(InitArgs),
    /// Respond simply
    Do(DoArgs),
    /// Be creative
    #[command(name = "invent")]
    Invent(InventArgs),
    /// Write code
    Code(crate::cli::code_flow::CodeArgs),
    /// Reason scientifically (prefer bare `malvin REQUEST`)
    #[command(hide = true)]
    Kpop(KpopArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// List available models
    Models(ModelsArgs),
    /// Reflect on and revise a plan file
    Plan(PlanArgs),
}
