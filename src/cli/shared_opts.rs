//! Shared CLI flags (`SharedOpts`) are parsed globally for every subcommand. `model`, `no_force`, `no_tenacious`, `no_tee`, and `max_acp_retries` affect `malvin code`, `malvin kpop`, `malvin invent`, and `malvin do`. `--verbose` logs full outgoing agent prompts to stdout and `prompts.log` (default is prompt name only). `--no-markdown` disables styled ACP stdout for subcommands that use `acp_stdout_markdown_enabled()` (`code`, `kpop`, `tidy` when the agent runs, `invent`, `init` summary, and `do` on a TTY). It is a no-op for `models` (no agent). Piped `malvin do` output stays plain regardless of `--no-markdown`.

pub use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::Args;

const NO_TEE_HELPTEXT: &str = "Omit stdout streaming [default: tee on].";
const NO_MARKDOWN_HELPTEXT: &str = "Disable styled markdown";

/// Flags that apply to every subcommand (place before or after the subcommand name).
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Turn off color output.
    #[arg(long, global = true, default_value_t = false)]
    pub no_color: bool,
    /// Suppress all stdout (run logs under `.malvin/logs` are unchanged).
    #[arg(short = 'b', long, global = true, default_value_t = false)]
    pub background: bool,
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
    /// Don't expand gate-loop budgets to tenacious limits [default: tenacious on].
    #[arg(long = "no-tenacious", global = true, default_value_t = false)]
    pub no_tenacious: bool,
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
    /// Max bounded attempts per ACP spawn or `session/prompt` (1s / 3s backoff between tries).
    #[arg(long = "max-acp-retries", global = true, default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Print built-in documentation (`malvin --doc` or `malvin <COMMAND> --doc`) and exit.
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !self.no_tee && !crate::output::stdout_suppressed()
    }

    #[must_use]
    pub(crate) const fn acp_stdout_markdown_enabled(&self) -> bool {
        !self.no_markdown
    }
}
