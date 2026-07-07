//! CLI argument structs for the `malvin` binary.

use clap::{Parser, Subcommand};

use super::shared_opts::SharedOpts;
use super::delight_flow::DelightArgs;
use super::explain_flow::ExplainArgs;
use super::revise_flow::ReviseArgs;
use super::tidy_flow::TidyArgs;

pub use super::logs_cmd::LogsArgs;
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
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Respond simply
    Do(DoArgs),
    /// Be creative
    #[command(name = "inspire")]
    Inspire(InspireArgs),
    /// Write code
    Code(crate::cli::code_flow::CodeArgs),
    /// Reason scientifically
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
}
