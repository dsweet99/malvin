use clap::{ArgAction, Args};
use malvin::config::malvin_config_file::parse_model_cli_arg;
use malvin::config::model_id::ParsedModel;
pub use malvin::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};

const QUIET_HELPTEXT: &str =
    "Print only `__MALVIN_DM_START__`/`END` bodies on stdout (default router)";

const CREATIVE_HELPTEXT: &str = "Be (more) creative for the REQUEST that immediately follows; optional probability in [0,1] (default 1.0 when set; repeatable)";

const WATCH_HELPTEXT: &str =
    "Re-copy the request `.md` into the run log dir before each outer loop (overwrite)";

const ML_HELPTEXT: &str = "Run the meta-loop N times (positive integer, or inf to repeat forever)";

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

pub(crate) fn parse_creative_probability(s: &str) -> Result<f64, String> {
    let p: f64 = s
        .parse()
        .map_err(|e| format!("invalid --creative probability `{s}`: {e}"))?;
    if !(0.0..=1.0).contains(&p) || !p.is_finite() {
        return Err(format!(
            "--creative probability must be a finite value in [0.0, 1.0], got {p}"
        ));
    }
    Ok(p)
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
    /// Log full outgoing agent prompts to stdout and `prompts.log`
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
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
    /// Print only `__MALVIN_DM_START__`/`END` bodies on stdout (default router)
    #[arg(short = 'q', long, default_value_t = false, help = QUIET_HELPTEXT)]
    pub quiet: bool,
    /// Run workspace quality gates; treat failures as loop or exit criteria
    #[arg(short = 'g', long, default_value_t = false)]
    pub gates: bool,
    /// Be (more) creative for the REQUEST that immediately follows (repeatable)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "1.0",
        require_equals = true,
        value_name = "PROB",
        value_parser = parse_creative_probability,
        action = ArgAction::Append,
        help = CREATIVE_HELPTEXT
    )]
    pub creative: Vec<f64>,
    /// Re-copy the request `.md` into the run log dir before each outer loop
    #[arg(long, default_value_t = false, help = WATCH_HELPTEXT)]
    pub watch: bool,
}

impl SharedOpts {
    #[must_use]
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !malvin::output::stdout_suppressed()
    }

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
    pub(crate) fn tee_startup_stdout(&self) -> bool {
        !self.quiet && !malvin::output::stdout_suppressed()
    }

    #[must_use]
    pub(crate) fn creative_probability(&self) -> Option<f64> {
        self.creative.last().copied()
    }

    #[must_use]
    pub(crate) fn with_creative_probability(mut self, creative: Option<f64>) -> Self {
        self.creative = creative.map_or_else(Vec::new, |p| vec![p]);
        self
    }

    #[must_use]
    pub(crate) fn sample_creative_this_iteration(&self) -> bool {
        match self.creative_probability() {
            None => false,
            Some(p) if p <= 0.0 => false,
            Some(p) if p >= 1.0 => true,
            Some(p) => fastrand::f64() < p,
        }
    }
}

#[cfg(test)]
impl SharedOpts {
    #[must_use]
    pub(crate) fn test_defaults() -> Self {
        Self {
            model: malvin::config::model_id::parse_model_id(malvin::config::DEFAULT_CLI_MODEL)
                .expect("default model"),
            verbose: false,
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
            quiet: false,
            gates: false,
            creative: Vec::new(),
            watch: false,
        }
    }
}

#[cfg(test)]
mod overlay_tests {
    #[test]
    fn creative_flag_defaults_off_and_accepts_probability() {
        use clap::Parser;
        let off = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert!(off.router.creative_probability().is_none());
        assert!(!off.router.sample_creative_this_iteration());

        let on = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"]).expect("parse");
        assert_eq!(on.router.creative_probability(), Some(1.0));
        assert!(on.router.sample_creative_this_iteration());

        let p =
            crate::cli::Cli::try_parse_from(["malvin", "--creative=0.6", "--doc"]).expect("parse");
        assert_eq!(p.router.creative_probability(), Some(0.6));

        let zero =
            crate::cli::Cli::try_parse_from(["malvin", "--creative=0", "--doc"]).expect("parse");
        assert_eq!(zero.router.creative_probability(), Some(0.0));
        assert!(!zero.router.sample_creative_this_iteration());

        let multi = crate::cli::Cli::try_parse_from([
            "malvin",
            "--creative",
            "a",
            "--creative=0.5",
            "b",
            "--doc",
        ])
        .expect("parse");
        assert_eq!(multi.router.creative, vec![1.0, 0.5]);
    }

    #[test]
    fn creative_probability_rejects_out_of_range() {
        assert!(super::parse_creative_probability("1.1").is_err());
        assert!(super::parse_creative_probability("-0.1").is_err());
        assert!(super::parse_creative_probability("nope").is_err());
        assert_eq!(super::parse_creative_probability("0.5").ok(), Some(0.5));
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
}
