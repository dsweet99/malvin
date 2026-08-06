//! Shared CLI flags (`SharedOpts`) are parsed globally for every subcommand. `model`, `no_force`, `no_tenacious`, and `max_acp_retries` affect `malvin inspire` and `malvin --do`. `--gates` enables harness-run quality gates for the default route; `malvin tidy` always forces them on. `--verbose` logs full outgoing agent prompts to stdout and `prompts.log` (default is prompt name only). For `malvin --do`, `--verbose` also unlocks the same live agent log classes as the default workflow (thought tokens and narrative tee); without `--verbose`, `--do` stays DM-body-only. `--quiet` / `-q` restricts default-router stdout (bare `malvin REQUEST`, `tidy`, `explain`) to `MALVIN_DM_*` bodies only. `--git` sets `{{ git_extra }}` so prompt templates may permit `git commit` (default off).

pub use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::Args;

const QUIET_HELPTEXT: &str =
    "Stdout: only MALVIN_DM_START/END bodies (default workflow; not -b)";

/// Flags that apply to every subcommand (place before or after the subcommand name).
#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Suppress all stdout.
    #[arg(short = 'b', long, global = true, default_value_t = false)]
    pub background: bool,
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SharedOpts {
    /// Model id (`cursor:`, `prime:`, or `mini:`).
    #[arg(long, global = true, default_value = DEFAULT_CLI_MODEL)]
    pub model: String,
    /// Don't force Cursor SDK tool auto-run (fails fast on `cursor:`; SDK has no interactive approval).
    #[arg(long, global = true, default_value_t = false)]
    pub no_force: bool,
    /// Don't expand gate-loop budgets to tenacious limits [default: tenacious on].
    #[arg(long = "no-tenacious", global = true, default_value_t = false)]
    pub no_tenacious: bool,
    /// Run workspace quality gates in the harness and treat failures as loop/exit criteria.
    #[arg(long, global = true, default_value_t = false)]
    pub gates: bool,
    /// Restrict process stdout to `MALVIN_DM_START`/`MALVIN_DM_END` bodies on the default router.
    #[arg(
        short = 'q',
        long,
        global = true,
        default_value_t = false,
        help = QUIET_HELPTEXT
    )]
    pub quiet: bool,
    /// Log full outgoing agent prompt bodies to stdout and `prompts.log` (default: prompt name only).
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,
    /// Max agent retries per spawn, HTTP completion, or gate iteration (1s / 3s backoff between tries).
    #[arg(long = "max-acp-retries", global = true, default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Deprecated alias for `--mini-max-http-turns`.
    #[arg(long = "mini-max-bash-turns", global = true, default_value_t = 32, hide = true)]
    pub mini_max_bash_turns: u32,
    /// Max Investigate-phase HTTP turns per `run_coder_prompt` for `mini:` models [default: 32].
    #[arg(long = "mini-max-http-turns", global = true, default_value_t = 32, hide = true)]
    pub mini_max_http_turns: u32,
    /// Max bash subprocess executions per `run_coder_prompt` for `mini:` models [default: 128].
    #[arg(long = "mini-max-bash-execs", global = true, default_value_t = 128, hide = true)]
    pub mini_max_bash_execs: u32,
    /// Max transient HTTP retries per completion for `mini:` models [default: 0].
    #[arg(long = "mini-max-http-retries", global = true, default_value_t = 0, hide = true)]
    pub mini_max_http_retries: u32,
    /// Max transport-layer retries per HTTP completion for `mini:` models (from config when unset).
    #[arg(skip)]
    pub mini_max_transport_retries: u32,
    /// Max whole-loop gate retries after failure for `mini:` models [default: 0].
    #[arg(long = "mini-max-gate-retries", global = true, default_value_t = 0, hide = true)]
    pub mini_max_gate_retries: u32,
    /// Max context-recovery shrink passes per overflow for `mini:` models [default: 0].
    #[arg(long = "mini-max-shrink-passes", global = true, default_value_t = 0, hide = true)]
    pub mini_max_shrink_passes: u32,
    /// Do not auto-download `mini:local/…` models on first use (fail if missing from cache).
    #[arg(long = "no-download", global = true, default_value_t = false)]
    pub no_download: bool,
    /// Print built-in documentation (`malvin --doc` or `malvin <COMMAND> --doc`) and exit.
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Session name for this malvin process (default: random five-character id).
    #[arg(long, global = true)]
    pub name: Option<String>,
    /// Allow the agent to run `git commit` (sets `{{ git_extra }}` in prompt templates).
    #[arg(long, global = true, default_value_t = false)]
    pub git: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !self.quiet && !crate::output::stdout_suppressed()
    }

    /// Styled ACP stdout markdown is always enabled for agent-backed flows (TTY gates apply downstream).
    #[must_use]
    pub(crate) const fn acp_stdout_markdown_enabled(&self) -> bool {
        true
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
            gates: false,
            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            mini_max_bash_turns: 32,
            mini_max_http_turns: 32,
            mini_max_bash_execs: 128,
            mini_max_http_retries: 0,
            mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
            mini_max_gate_retries: 0,
            mini_max_shrink_passes: 0,
            no_download: false,
            git: false,
        }
    }
}
