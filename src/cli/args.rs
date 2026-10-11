use clap::{Parser, Subcommand};

use super::admin_cmd::AdminArgs;
use super::request_argv::TaggedRequest;
use super::shared_opts::{RouterOpts, SharedOpts};

#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(
    name = "malvin",
    version,
    about = "Non-interactive research and coding agent",
    disable_help_subcommand = true,
    override_usage = "malvin [OPTION]... [REQUEST]...\n   or: malvin [OPTION]... <COMMAND>",
)]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedOpts,
    #[command(flatten)]
    pub router: RouterOpts,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text.
    /// Multiple values each run as an independent invocation.
    #[arg(value_name = "REQUEST")]
    pub requests: Vec<String>,
    /// Filled after clap parse: per-REQUEST workflow tags (see `request_argv`).
    #[arg(skip)]
    pub tagged_requests: Vec<TaggedRequest>,
}

impl Cli {
    #[must_use]
    pub const fn has_request(&self) -> bool {
        !self.requests.is_empty()
    }

    #[cfg(test)]
    #[must_use]
    pub fn first_request(&self) -> Option<&String> {
        self.requests.first()
    }

    #[must_use]
    pub fn has_router_request(&self) -> bool {
        if !self.tagged_requests.is_empty() {
            return self.tagged_requests.iter().any(TaggedRequest::is_router);
        }
        self.has_request()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Operator maintenance commands
    Admin(AdminArgs),
}
