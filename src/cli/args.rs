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
    override_usage = "malvin [OPTION]... [REQUEST]...\n   or: malvin [OPTION]... <COMMAND>",
    after_help = "Bare malvin REQUEST runs autonomous routing. Multiple REQUEST args each run independently (new log dir, full router loop, summarize). Use `--do` for a one-shot turn, or subcommands for named workflows."
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
    /// Existing `.md` path or literal text (bare malvin REQUEST, or request for `--do`).
    /// Multiple values each run as an independent invocation.
    #[arg(value_name = "REQUEST")]
    pub requests: Vec<String>,
}

impl Cli {
    #[must_use]
    pub const fn has_request(&self) -> bool {
        !self.requests.is_empty()
    }

    #[must_use]
    pub fn first_request(&self) -> Option<&String> {
        self.requests.first()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Operator maintenance commands
    Admin(AdminArgs),
}
