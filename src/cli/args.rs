//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::revise_flow::ReviseArgs;
use super::tidy_flow::TidyArgs;

pub use super::generate_script_cmd::GenerateScriptArgs;
pub use super::logs_cmd::LogsArgs;
pub use super::models_cmd::ModelsArgs;
pub use crate::do_flow::DoArgs;
pub use crate::inspire_flow::InspireArgs;
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
    after_help = "Bare invocation: malvin REQUEST... runs kpop on each request in sequence (same as malvin kpop REQUEST). Use subcommands for do, code, and tidy."
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
    /// When no subcommand: kpop request text or path(s).
    #[arg(value_name = "REQUEST", num_args = 0..)]
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
    #[command(name = "inspire")]
    Inspire(InspireArgs),
    /// Write code
    Code(crate::cli::code_flow::CodeArgs),
    /// Reason scientifically (prefer bare `malvin REQUEST`)
    #[command(hide = true)]
    Kpop(KpopArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// Author a user-delighting feature pitch
    Delight(DelightArgs),
    /// Explain code or concepts via LaTeX PDF
    Explain(ExplainArgs),
    /// Revise a document in place
    Revise(ReviseArgs),
    /// List available models
    Models(ModelsArgs),
    /// Inspect and manage run-log retention
    Logs(LogsArgs),
    /// Generate auto-script JSON and stub shell scripts from a recipe
    #[command(name = "generate-script")]
    GenerateScript(GenerateScriptArgs),
}
