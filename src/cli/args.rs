use clap::{Parser, Subcommand};

use super::admin_cmd::AdminArgs;
use super::shared_opts::{RouterOpts, SharedOpts};

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
    #[command(flatten)]
    pub router: RouterOpts,
    /// One-shot agent turn (non-looping)
    #[arg(long = "do", default_value_t = false)]
    pub do_workflow: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text (bare malvin REQUEST, or request for `--do`)
    pub request: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Operator maintenance commands
    Admin(AdminArgs),
}
