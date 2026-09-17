use clap::{ArgAction, Parser, Subcommand};

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
    after_help = "Bare malvin REQUEST runs autonomous routing. Multiple REQUEST args each run independently (new log dir, full router loop, summarize). Each `--do` and each `--creative` applies only to the REQUEST that immediately follows it (repeatable). Other REQUEST args use the router without those prefixes. `--iml` (the Infinite Meta-Loop) cycles through all REQUEST args forever."
)]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedOpts,
    #[command(flatten)]
    pub router: RouterOpts,
    /// One-shot agent turn for the REQUEST that immediately follows (repeatable)
    #[arg(long = "do", action = ArgAction::Count, id = "do_workflow")]
    do_count: u8,
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// Existing `.md` path or literal text (bare malvin REQUEST, or request after `--do`).
    /// Multiple values each run as an independent invocation.
    #[arg(value_name = "REQUEST")]
    pub requests: Vec<String>,
    /// Filled after clap parse: per-REQUEST workflow tags (see `request_argv`).
    #[arg(skip)]
    pub tagged_requests: Vec<TaggedRequest>,
}

impl Cli {
    #[must_use]
    pub const fn do_workflow(&self) -> bool {
        self.do_count > 0
    }

    #[must_use]
    pub const fn has_request(&self) -> bool {
        !self.requests.is_empty()
    }

    #[must_use]
    pub fn first_request(&self) -> Option<&String> {
        self.requests.first()
    }

    #[must_use]
    pub fn has_router_request(&self) -> bool {
        if !self.tagged_requests.is_empty() {
            return self.tagged_requests.iter().any(TaggedRequest::is_router);
        }
        !self.do_workflow() && self.has_request()
    }

    #[must_use]
    pub fn has_do_request(&self) -> bool {
        if !self.tagged_requests.is_empty() {
            return self.tagged_requests.iter().any(TaggedRequest::is_do);
        }
        self.do_workflow() && self.has_request()
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Operator maintenance commands
    Admin(AdminArgs),
}
