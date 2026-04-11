//! Shared CLI flags for `code` and `kpop`.

use clap::Args;

#[derive(Args, Debug)]
pub struct SharedOpts {
    /// Model label.
    #[arg(long, default_value = "opus-4.5")]
    pub model: String,
    /// Disable force-mode (omit `agent --force`).
    #[arg(long, default_value_t = false)]
    pub no_force: bool,
    /// Disable tee: do not echo the plan/request, the startup `Command:` line, or ACP session output to stdout [default: tee on]. Matches the tee contract in `grounding.md`. Run-directory files (for example `command.log` and trace logs) are always written.
    #[arg(long, default_value_t = false)]
    pub no_tee: bool,
}

impl SharedOpts {
    /// Whether to echo the plan/request and startup `Command:` line to stdout before agent work (`--no-tee` disables). Same `no_tee` flag is passed to the agent for ACP log tee; see [`malvin::agent::AgentIoOptions`].
    #[must_use]
    pub(crate) const fn tee_startup_stdout(&self) -> bool {
        !self.no_tee
    }
}
