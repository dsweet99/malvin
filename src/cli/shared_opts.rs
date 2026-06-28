//! Shared CLI flags (`SharedOpts`) are parsed globally for every subcommand. `model`, `no_force`, `no_tenacious`, `no_tee`, and `max_acp_retries` affect `malvin code`, `malvin kpop`, `malvin inspire`, and `malvin do`. `--verbose` logs full outgoing agent prompts to stdout and `prompts.log` (default is prompt name only). `--no-markdown` disables styled ACP stdout for subcommands that use `acp_stdout_markdown_enabled()` (`code`, `kpop`, `tidy` when the agent runs, `inspire`, `init` summary, and `do` on a TTY). It is a no-op for `models` (no agent). Piped `malvin do` output stays plain regardless of `--no-markdown`.

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
    /// Max agent retries per spawn, HTTP completion, or gate iteration (1s / 3s backoff between tries).
    #[arg(long = "max-acp-retries", global = true, default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Use in-process mini agent (`OpenRouter` + bash loop) instead of Cursor ACP.
    #[arg(long, global = true, default_value_t = false)]
    pub mini: bool,
    /// Deprecated alias for `--mini-max-http-turns`.
    #[arg(long = "mini-max-bash-turns", global = true, default_value_t = 32, hide = true)]
    pub mini_max_bash_turns: u32,
    /// Max Investigate-phase HTTP turns per `run_coder_prompt` when `--mini` [default: 32].
    #[arg(long = "mini-max-http-turns", global = true, default_value_t = 32)]
    pub mini_max_http_turns: u32,
    /// Max bash subprocess executions per `run_coder_prompt` when `--mini` [default: 128].
    #[arg(long = "mini-max-bash-execs", global = true, default_value_t = 128)]
    pub mini_max_bash_execs: u32,
    /// Max transient `OpenRouter` HTTP retries per completion when `--mini` [default: 0].
    #[arg(long = "mini-max-http-retries", global = true, default_value_t = 0)]
    pub mini_max_http_retries: u32,
    /// Max transport-layer retries per `OpenRouter` completion when `--mini` (from config when unset).
    #[arg(skip)]
    pub mini_max_transport_retries: u32,
    /// Max whole-loop gate retries after failure when `--mini` [default: 0].
    #[arg(long = "mini-max-gate-retries", global = true, default_value_t = 0)]
    pub mini_max_gate_retries: u32,
    /// Max context-recovery shrink passes per overflow when `--mini` [default: 0].
    #[arg(long = "mini-max-shrink-passes", global = true, default_value_t = 0)]
    pub mini_max_shrink_passes: u32,
    /// Print built-in documentation (`malvin --doc` or `malvin <COMMAND> --doc`) and exit.
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Session name for this malvin process (default: random five-character id).
    #[arg(long, global = true)]
    pub name: Option<String>,
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

#[cfg(test)]
impl SharedOpts {
    #[must_use]
    pub(crate) fn test_defaults() -> Self {
        Self {
            model: crate::config::DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tenacious: false,
            no_tee: true,
            no_markdown: true,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            mini: false,
            mini_max_bash_turns: 32,
            mini_max_http_turns: 32,
            mini_max_bash_execs: 128,
            mini_max_http_retries: 0,
            mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
            mini_max_gate_retries: 0,
            mini_max_shrink_passes: 0,
        }
    }
}
