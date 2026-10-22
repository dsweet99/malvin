use clap::{ArgAction, Args};
use malvin::config::malvin_config_file::{DEFAULT_MAX_LOOPS, parse_model_cli_arg};
use malvin::config::model_id::ParsedModel;
pub use malvin::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

const QUIET_HELPTEXT: &str = "Only print DMs";

const VERY_QUIET_HELPTEXT: &str = "Print nothing (-qq)";

const ML_HELPTEXT: &str = "Run the meta-loop N times (positive integer, or inf to repeat forever)";

const MAX_LOOPS_HELPTEXT: &str =
    "Outer-loop budget for the default route and gates-only runs (positive integer)";

pub(crate) fn parse_max_loops(s: &str) -> Result<usize, String> {
    s.parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("--max-loops must be a positive integer, got `{s}`"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetaLoopCount {
    Times(u64),
    Forever,
}

impl Default for MetaLoopCount {
    fn default() -> Self {
        Self::Times(1)
    }
}

impl std::fmt::Display for MetaLoopCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Times(n) => write!(f, "{n}"),
            Self::Forever => write!(f, "inf"),
        }
    }
}

impl MetaLoopCount {
    #[must_use]
    pub(crate) const fn is_forever(self) -> bool {
        matches!(self, Self::Forever)
    }
}

pub(crate) fn parse_meta_loop_count(s: &str) -> Result<MetaLoopCount, String> {
    if s == "inf" {
        return Ok(MetaLoopCount::Forever);
    }
    let n: u64 = s
        .parse()
        .map_err(|_| format!("--ml must be a positive integer or inf, got `{s}`"))?;
    if n == 0 {
        return Err(format!("--ml must be a positive integer or inf, got `{s}`"));
    }
    Ok(MetaLoopCount::Times(n))
}

/// Options shared across workflows (model, logging, retries, docs).
#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SharedOpts {
    /// Model id (`cursor:`, `pi:`, `codex:`, or an `[aliases.models]` name)
    #[arg(
        long,
        default_value = DEFAULT_CLI_MODEL,
        value_parser = parse_model_cli_arg
    )]
    pub model: ParsedModel,
    /// Do not ask a nested malvin run to configure missing token costs
    #[arg(long, default_value_t = false)]
    pub no_auto_costs: bool,
    /// Stop after N consecutive identical backend errors, each within 60 s of the previous one
    #[arg(long = "max-acp-retries", default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Print built-in documentation and exit
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Print full advice for TAG, or list tags (tag then description) if TAG is omitted (no run logs)
    #[arg(long, value_name = "TAG", num_args = 0..=1, default_missing_value = "")]
    pub advice: Option<String>,
    /// Print credits for ideas malvin builds on and exit (no run logs)
    #[arg(long, default_value_t = false)]
    pub credits: bool,
    /// Run every REQUEST this many times (as if re-invoking the same command line)
    #[arg(
        long = "ml",
        value_name = "N",
        default_value_t = MetaLoopCount::Times(1),
        value_parser = parse_meta_loop_count,
        help = ML_HELPTEXT
    )]
    pub ml: MetaLoopCount,
    /// Run on a remote machine, as PROVIDER:SERVICE[KEY=VALUE,...] (see `malvin admin remotes`; `[aliases.remotes]` names work too)
    #[arg(
        long,
        value_name = "REMOTE",
        value_parser = [malvin::modal_run::MODAL_REMOTE_ID],
        hide_possible_values = true
    )]
    pub remote: Option<String>,
}

/// Options that apply only to default-route / gates-only loops.
#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RouterOpts {
    /// Only print DMs. Repeat the short flag (`-qq`) to print nothing.
    #[arg(short = 'q', long, action = ArgAction::Count, help = QUIET_HELPTEXT)]
    pub quiet: u8,
    /// Print nothing (`-qq` is the short form)
    #[arg(long = "very-quiet", default_value_t = false, help = VERY_QUIET_HELPTEXT)]
    pub very_quiet: bool,
    /// Run workspace quality gates; treat failures as loop or exit criteria
    #[arg(short = 'g', long, default_value_t = false)]
    pub gates: bool,
    /// Outer-loop budget for the default route and gates-only runs
    #[arg(
        long = "max-loops",
        default_value_t = DEFAULT_MAX_LOOPS,
        value_parser = parse_max_loops,
        help = MAX_LOOPS_HELPTEXT
    )]
    pub max_loops: usize,
}

impl SharedOpts {
    #[must_use]
    pub(crate) const fn acp_stdout_markdown_enabled(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AgentRouteOpts<'a> {
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
}

impl RouterOpts {
    #[must_use]
    pub(crate) const fn dm_stdout_only(&self) -> bool {
        self.quiet == 1 && !self.very_quiet
    }

    #[must_use]
    pub(crate) const fn prints_nothing(&self) -> bool {
        self.very_quiet || self.quiet >= 2
    }

    #[must_use]
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !self.dm_stdout_only() && !self.prints_nothing() && !malvin::output::stdout_suppressed()
    }
}

#[cfg(test)]
impl SharedOpts {
    #[must_use]
    pub(crate) fn test_defaults() -> Self {
        Self {
            model: malvin::config::model_id::parse_model_id(malvin::config::DEFAULT_CLI_MODEL)
                .expect("default model"),
            no_auto_costs: false,
            max_acp_retries: malvin::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            advice: None,
            credits: false,
            ml: MetaLoopCount::Times(1),
            remote: None,
        }
    }
}

#[cfg(test)]
impl RouterOpts {
    #[must_use]
    pub(crate) fn test_defaults() -> Self {
        Self {
            quiet: 0,
            very_quiet: false,
            gates: false,
            max_loops: DEFAULT_MAX_LOOPS,
        }
    }
}

#[cfg(test)]
mod overlay_tests {
    #[test]
    fn creative_flag_is_rejected() {
        use clap::Parser;
        let err = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"])
            .expect_err("removed flag");
        let msg = err.to_string();
        assert!(
            msg.contains("unexpected") || msg.contains("unknown") || msg.contains("--creative"),
            "{msg}"
        );
        assert!(crate::cli::Cli::try_parse_from(["malvin", "--creative=0.6", "task"]).is_err());
    }

    #[test]
    fn ml_flag_defaults_to_one_and_parses_count_or_inf() {
        use clap::Parser;
        let off = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert_eq!(off.shared.ml, super::MetaLoopCount::Times(1));

        let thrice = crate::cli::Cli::try_parse_from(["malvin", "--ml=3", "--doc"]).expect("parse");
        assert_eq!(thrice.shared.ml, super::MetaLoopCount::Times(3));

        let forever =
            crate::cli::Cli::try_parse_from(["malvin", "--ml=inf", "--doc"]).expect("parse");
        assert!(forever.shared.ml.is_forever());

        assert!(crate::cli::Cli::try_parse_from(["malvin", "--ml=0", "--doc"]).is_err());
        assert!(crate::cli::Cli::try_parse_from(["malvin", "--ml=nope", "--doc"]).is_err());
        assert!(crate::cli::Cli::try_parse_from(["malvin", "--iml", "--doc"]).is_err());
    }

    #[test]
    fn help_lists_ml_count() {
        use clap::CommandFactory;
        let help = crate::cli::Cli::command().render_help().to_string();
        assert!(help.contains("--ml"), "help={help}");
        assert!(!help.contains("--iml"), "help={help}");
        assert!(
            help.contains("inf"),
            "help must say inf repeats forever; got {help}"
        );
    }

    #[test]
    fn help_lists_quiet_and_very_quiet() {
        use clap::CommandFactory;
        let help = crate::cli::Cli::command().render_help().to_string();
        assert!(
            help.contains("Only print DMs"),
            "quiet help must say Only print DMs; got {help}"
        );
        assert!(
            !help.contains("bodies on stdout"),
            "quiet help must drop the old wording; got {help}"
        );
        assert!(
            help.contains("--very-quiet"),
            "help must list --very-quiet; got {help}"
        );
        assert!(
            help.contains("Print nothing (-qq)"),
            "very-quiet help must say Print nothing (-qq); got {help}"
        );
    }
}
