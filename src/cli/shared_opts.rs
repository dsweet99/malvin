pub use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::Args;

use crate::model_id::{parse_model_id, ParsedModel};

const QUIET_HELPTEXT: &str =
    "Print only `MALVIN_DM_START`/`END` bodies on stdout (default router; not `-b`)";

#[derive(Args, Debug)]
pub struct GlobalOpts {
    /// Suppress all stdout
    #[arg(short = 'b', long, global = true, default_value_t = false)]
    pub background: bool,
}

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SharedOpts {
    /// Model id (`cursor:` or `pi:`)
    #[arg(
        long,
        global = true,
        default_value = DEFAULT_CLI_MODEL,
        value_parser = parse_model_id
    )]
    pub model: ParsedModel,
    /// Do not auto-approve tool calls (unsupported on `cursor:` and `pi:`; fails fast)
    #[arg(long, global = true, default_value_t = false)]
    pub no_force: bool,
    /// Do not expand gate-loop budgets to tenacious limits (tenacious on by default)
    #[arg(long = "no-tenacious", global = true, default_value_t = false)]
    pub no_tenacious: bool,
    /// Run workspace quality gates; treat failures as loop or exit criteria
    #[arg(short = 'g', long, global = true, default_value_t = false)]
    pub gates: bool,
    /// Print only `MALVIN_DM_START`/`END` bodies on stdout (default router; not `-b`)
    #[arg(
        short = 'q',
        long,
        global = true,
        default_value_t = false,
        help = QUIET_HELPTEXT
    )]
    pub quiet: bool,
    /// Log full outgoing agent prompts to stdout and `prompts.log`
    #[arg(short, long, global = true, default_value_t = false)]
    pub verbose: bool,
    /// Max agent retries per spawn or gate iteration
    #[arg(long = "max-acp-retries", global = true, default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Print built-in documentation and exit
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Session name for bare malvin REQUEST, `--do`, `tidy`, and `init` (default: random five-character id)
    #[arg(long, global = true)]
    pub name: Option<String>,
    /// Allow the agent to run `git commit`
    #[arg(long, global = true, default_value_t = false)]
    pub git: bool,
    /// Use the creative `router_b` prompt on the default router
    #[arg(long, global = true, default_value_t = false)]
    pub creative: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !self.quiet && !crate::output::stdout_suppressed()
    }

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
            git: false,
            creative: false,
        }
    }
}
