//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::init_flow::InitArgs;
use super::tidy_flow::TidyArgs;

pub use super::models_cmd::ModelsArgs;
pub use crate::inspire_flow::InspireArgs;
pub use super::shared_opts::GlobalOpts;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive research and coding agent",
    disable_help_subcommand = true,
    after_help = "Bare `malvin REQUEST` runs autonomous routing. Use `--do` for a one-shot turn, or subcommands for named workflows."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(flatten)]
    pub shared: SharedOpts,
    /// Respond simply (one-shot agent turn).
    #[arg(long = "do", default_value_t = false)]
    pub do_workflow: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text (bare `malvin REQUEST`, or request for `--do`).
    pub request: Option<String>,
    /// Outer agent-session budget for bare `malvin REQUEST` (`effective_max_loops`).
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS)]
    pub max_loops: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Discover quality gates and write `.malvin/checks`
    Init(InitArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// Explain code or concepts via LaTeX PDF
    Explain(ExplainArgs),
    /// Be creative
    #[command(name = "inspire")]
    Inspire(InspireArgs),
    /// Be creative (legacy name)
    #[command(name = "adaptix", hide = true)]
    Adaptix(InspireArgs),
    /// Write code (deprecated; hidden from help)
    #[command(hide = true)]
    Code(crate::cli::code_flow::CodeArgs),
    /// Author a user-delighting feature pitch
    #[command(hide = true)]
    Delight(DelightArgs),
    /// List available models
    Models(ModelsArgs),
}
