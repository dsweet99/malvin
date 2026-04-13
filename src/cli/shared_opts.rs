//! Shared CLI flags for `code`, `kpop`, and `do`, plus root-level global flags.

use clap::Args;

/// Default for [`SharedOpts::model`] when `--model` is omitted.
pub const DEFAULT_CLI_MODEL: &str = "composer-2";

/// Flags that apply to every subcommand (place before or after the subcommand name).
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// No ANSI on stdout prefixes (time, `[who]`).
    #[arg(long, global = true, default_value_t = false)]
    pub no_color: bool,
}

#[derive(Args, Debug)]
pub struct SharedOpts {
    /// Model id.
    #[arg(long, default_value = DEFAULT_CLI_MODEL)]
    pub model: String,
    /// Omit `agent --force`.
    #[arg(long, default_value_t = false)]
    pub no_force: bool,
    /// Omit stdout tee: plan echo, `Command:` line, and ACP session log [default: tee on].
    #[arg(long, default_value_t = false)]
    pub no_tee: bool,
}

impl SharedOpts {
    /// Echo plan and startup `Command:` to stdout before agent work; `--no-tee` disables. Same flag controls ACP log tee ([`malvin::acp::AgentIoOptions`]).
    #[must_use]
    pub(crate) const fn tee_startup_stdout(&self) -> bool {
        !self.no_tee
    }
}
