//! Shared CLI flags for `code`, `kpop`, and `do`, plus root-level global flags.

use clap::Args;
pub use malvin::config::DEFAULT_CLI_MODEL;

const NO_TEE_HELPTEXT: &str = "Omit stdout streaming [default: tee on].";

/// Flags that apply to every subcommand (place before or after the subcommand name).
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Turn off color output.
    #[arg(long, global = true, default_value_t = false)]
    pub no_color: bool,
}

#[derive(Args, Debug)]
pub struct SharedOpts {
    /// Model id.
    #[arg(long, global = true, default_value = DEFAULT_CLI_MODEL)]
    pub model: String,
    /// Don't `--force` cursor-agent.
    #[arg(long, global = true, default_value_t = false)]
    pub no_force: bool,
    #[arg(
        long,
        global = true,
        default_value_t = false,
        help = NO_TEE_HELPTEXT
    )]
    pub no_tee: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) const fn tee_startup_stdout(&self) -> bool {
        !self.no_tee
    }
}
