//! Shared CLI flags (`SharedOpts`) are parsed globally for every subcommand. `model`, `no_force`, and `no_tee` affect `malvin code`, `malvin kpop`, `malvin bug`, and `malvin do`. `--verbose` logs full outgoing agent prompts to stdout and `prompts.log` (default is prompt name only). `--no-markdown` disables styled ACP stdout for subcommands that use `acp_stdout_markdown_enabled()` (`code`, `kpop`, `bug`, `plan`, `tidy` when the agent runs, `init` summary). It is a no-op for `models` (no agent). `malvin do` forces plain stdout regardless of `--no-markdown`.

use clap::Args;
pub use crate::config::DEFAULT_CLI_MODEL;

const NO_TEE_HELPTEXT: &str = "Omit stdout streaming [default: tee on].";
const NO_MARKDOWN_HELPTEXT: &str = "Disable styled markdown";

/// Flags that apply to every subcommand (place before or after the subcommand name).
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Turn off color output.
    #[arg(long, global = true, default_value_t = false)]
    pub no_color: bool,
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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
    #[arg(
        long,
        global = true,
        default_value_t = false,
        help = NO_MARKDOWN_HELPTEXT
    )]
    pub no_markdown: bool,
    /// Log full outgoing agent prompt bodies to stdout and `prompts.log` (default: prompt name only).
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) const fn tee_startup_stdout(&self) -> bool {
        !self.no_tee
    }

    #[must_use]
    pub(crate) const fn acp_stdout_markdown_enabled(&self) -> bool {
        !self.no_markdown
    }
}
