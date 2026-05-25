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
    Code(crate::cli::code_flow::CodeArgs),
    /// Reason scientifically (q&a, research, optimization, ...)
    Kpop(KpopArgs),
    /// Ensure all checks pass
    Tidy(TidyArgs),
    /// List available models
    Models(ModelsArgs),
}
