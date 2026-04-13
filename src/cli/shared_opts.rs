//! Shared CLI flags for `code`, `kpop`, and `do`.

use clap::Args;

/// Default for [`SharedOpts::model`] when `--model` is omitted (product plan §4).
pub const DEFAULT_CLI_MODEL: &str = "composer-2";

#[derive(Args, Debug)]
pub struct SharedOpts {
    /// Model label.
    #[arg(long, default_value = DEFAULT_CLI_MODEL)]
    pub model: String,
    /// Disable force-mode (omit `agent --force`).
    #[arg(long, default_value_t = false)]
    pub no_force: bool,
    /// Disable tee: do not echo the plan/request, the startup `Command:` line, or ACP session output to stdout [default: tee on]. Progress lines, `Logs: …`, and `DONE` still print to stdout. Run timing (`run_timing.json` and one stdout summary line) is emitted after the workflow body as described in `grounding.md`. Run-directory files (for example `command.log` and trace logs) are always written.
    #[arg(long, default_value_t = false)]
    pub no_tee: bool,
}

impl SharedOpts {
    /// Whether to echo the plan/request and startup `Command:` line to stdout before agent work (`--no-tee` disables). Same `no_tee` flag is passed to the agent for ACP log tee; see [`malvin::acp::AgentIoOptions`].
    #[must_use]
    pub(crate) const fn tee_startup_stdout(&self) -> bool {
        !self.no_tee
    }
}
