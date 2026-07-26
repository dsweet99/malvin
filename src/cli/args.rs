//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::priors_flow::PriorsArgs;
use super::revise_flow::ReviseArgs;
use super::init_flow::InitArgs;
use super::tidy_flow::TidyArgs;

pub use super::models_cmd::ModelsArgs;
pub use crate::do_flow::DoArgs;
pub use crate::inspire_flow::InspireArgs;
pub use super::args_bug_kpop::KpopArgs;
pub use super::shared_opts::GlobalOpts;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive CLI agent, via Cursor ACP",
    disable_help_subcommand = true,
    after_help = "Bare `malvin REQUEST` runs autonomous routing. Use subcommands for named workflows."
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(flatten)]
    pub shared: SharedOpts,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text (bare `malvin REQUEST` autonomous routing).
    pub request: Option<String>,
    /// Legacy no-op for bare `malvin REQUEST` (single-session route; kept for CLI compatibility).
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS)]
    pub max_loops: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Respond simply
    Do(DoArgs),
    /// Be creative
    #[command(name = "inspire")]
    Inspire(InspireArgs),
    /// Be creative (legacy name)
    #[command(name = "adaptix", hide = true)]
    Adaptix(InspireArgs),
    /// Write code (deprecated; hidden from help)
    #[command(hide = true)]
    Code(crate::cli::code_flow::CodeArgs),
    /// Reason scientifically
    Kpop(KpopArgs),
    /// Discover quality gates and write `.malvin/checks`
    Init(InitArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// Author a user-delighting feature pitch
    Delight(DelightArgs),
    /// Ground a request in good priors
    Priors(PriorsArgs),
    /// Explain code or concepts via LaTeX PDF
    Explain(ExplainArgs),
    /// Revise a document in place
    Revise(ReviseArgs),
    /// List available models
    Models(ModelsArgs),
}
