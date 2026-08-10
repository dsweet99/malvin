//! Shared CLI flags (`SharedOpts`) are parsed globally for every subcommand. `model`, `no_force`, `no_tenacious`, and `max_acp_retries` affect `malvin inspire` and `malvin --do`. `--gates` / `-g` enables harness-run quality gates for the default route; `malvin tidy` always forces them on. `--verbose` logs full outgoing agent prompts to stdout and `prompts.log` (default is prompt name only). For `malvin --do`, `--verbose` also unlocks the same live agent log classes as the default workflow (thought tokens and narrative tee); without `--verbose`, `--do` stays DM-body-only. `--quiet` / `-q` restricts default-router stdout (bare `malvin REQUEST`, `tidy`, `write`) to `MALVIN_DM_*` bodies only. `--git` sets `{{ git_extra }}` so prompt templates may permit `git commit` (default off).

pub use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::Args;

use crate::model_id::{parse_model_id, ParsedModel};

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
    /// Model id (`cursor:` or `prime:`).
    #[arg(
        long,
        global = true,
        default_value = DEFAULT_CLI_MODEL,
        value_parser = parse_model_id
    )]
    pub model: ParsedModel,
    /// Don't force tool auto-run (fails fast on `cursor:` / `prime:`; no interactive approval).
    #[arg(long, global = true, default_value_t = false)]
    pub no_force: bool,
    /// Don't expand gate-loop budgets to tenacious limits [default: tenacious on].
    #[arg(long = "no-tenacious", global = true, default_value_t = false)]
    pub no_tenacious: bool,
    /// Run workspace quality gates in the harness and treat failures as loop/exit criteria.
    #[arg(short = 'g', long, global = true, default_value_t = false)]
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
    /// Max agent retries per spawn or gate iteration (1s / 3s backoff between tries).
    #[arg(long = "max-acp-retries", global = true, default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Do not auto-download `prime:local/…` models on first use (fail if missing from cache).
    #[arg(long = "no-download", global = true, default_value_t = false)]
    pub no_download: bool,
    /// Print built-in documentation (`malvin --doc` or `malvin <COMMAND> --doc`) and exit.
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Session name for bare `malvin REQUEST`, `--do`, and `tidy` (default: random five-character id).
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
            model: parse_model_id(crate::config::DEFAULT_CLI_MODEL).expect("default model"),
            no_force: true,
            no_tenacious: false,
            gates: false,
            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            name: None,
            no_download: false,
            git: false,
        }
    }
}
