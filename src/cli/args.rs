use clap::{Parser, Subcommand};

use super::admin_cmd::AdminArgs;
use super::shared_opts::SharedOpts;
use super::write_flow::WriteArgs;

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive research and coding agent",
    disable_help_subcommand = true,
    override_usage = "malvin [OPTION]... [REQUEST]\n   or: malvin [OPTION]... <COMMAND>",
    after_help = "Bare malvin REQUEST runs autonomous routing. Use `--do` for a one-shot turn, or subcommands for named workflows."
)]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedOpts,
    /// One-shot agent turn (non-looping)
    #[arg(long = "do", default_value_t = false)]
    pub do_workflow: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text (bare malvin REQUEST, or request for `--do`)
    pub request: Option<String>,
    /// Outer agent-session budget for bare malvin REQUEST
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_LOOPS)]
    pub max_loops: usize,
    /// Hypothesis budget for bare malvin REQUEST
    #[arg(long, default_value_t = crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES)]
    pub max_hypotheses: usize,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Write a LaTeX PDF on code or concepts
    Write(WriteArgs),
    /// Operator maintenance commands
    Admin(AdminArgs),
}
