use clap::Args;
pub use malvin::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use rand::Rng;

use malvin::model_id::{ParsedModel, parse_model_id};

const QUIET_HELPTEXT: &str =
    "Print only `__MALVIN_DM_START__`/`END` bodies on stdout (default router)";

const CREATIVE_HELPTEXT: &str =
    "Be (more) creative; optional probability in [0,1] (default 1.0 when set)";

const WATCH_HELPTEXT: &str =
    "Re-copy the request `.md` into the run log dir before each outer loop (overwrite)";

const IML_HELPTEXT: &str = "the Infinite Meta-Loop";

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
    /// Model id (`cursor:`, `pi:`, `rpi:`, or `codex:`)
    #[arg(
        long,
        default_value = DEFAULT_CLI_MODEL,
        value_parser = parse_model_id
    )]
    pub model: ParsedModel,
    /// Log full outgoing agent prompts to stdout and `prompts.log`
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Stop after N consecutive identical backend errors (default 3)
    #[arg(long = "max-acp-retries", default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Print built-in documentation and exit
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Cycle through all REQUEST args forever (as if re-invoking the same command line)
    #[arg(long = "iml", default_value_t = false, help = IML_HELPTEXT)]
    pub iml: bool,
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
    /// Be (more) creative; optional probability in [0,1] (default 1.0 when set)
    #[arg(
        long,
        num_args = 0..=1,
        default_missing_value = "1.0",
        require_equals = true,
        value_name = "PROB",
        value_parser = parse_creative_probability,
        help = CREATIVE_HELPTEXT
    )]
    pub creative: Option<f64>,
    /// Re-copy the request `.md` into the run log dir before each outer loop
    #[arg(long, default_value_t = false, help = WATCH_HELPTEXT)]
    pub watch: bool,
    /// Turn off `KPop`
    #[arg(long = "no-kpop", default_value_t = false, hide = true)]
    pub no_kpop: bool,
    /// Outer agent-session budget for bare malvin REQUEST
    #[arg(long, default_value_t = malvin::malvin_config_file::DEFAULT_MAX_LOOPS)]
    pub max_loops: usize,
    /// Hypothesis budget for bare malvin REQUEST
    #[arg(long, default_value_t = malvin::malvin_config_file::DEFAULT_MAX_HYPOTHESES)]
    pub max_hypotheses: usize,
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
    pub(crate) fn sample_creative_this_iteration(&self) -> bool {
        match self.creative {
            None => false,
            Some(p) if p <= 0.0 => false,
            Some(p) if p >= 1.0 => true,
            Some(p) => rand::thread_rng().gen_bool(p),
        }
    }
}

#[cfg(test)]
impl SharedOpts {
    #[must_use]
    pub(crate) fn test_defaults() -> Self {
        Self {
            model: parse_model_id(malvin::config::DEFAULT_CLI_MODEL).expect("default model"),
            verbose: false,
            max_acp_retries: malvin::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            iml: false,
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
            creative: None,
            watch: false,
            no_kpop: false,
            max_loops: malvin::malvin_config_file::DEFAULT_MAX_LOOPS,
            max_hypotheses: malvin::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
        }
    }
}

#[cfg(test)]
mod overlay_tests {
    #[test]
    fn creative_flag_defaults_off_and_accepts_probability() {
        use clap::Parser;
        let off = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert!(off.router.creative.is_none());
        assert!(!off.router.sample_creative_this_iteration());

        let on = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"]).expect("parse");
        assert_eq!(on.router.creative, Some(1.0));
        assert!(on.router.sample_creative_this_iteration());

        let p =
            crate::cli::Cli::try_parse_from(["malvin", "--creative=0.6", "--doc"]).expect("parse");
        assert_eq!(p.router.creative, Some(0.6));

        let zero =
            crate::cli::Cli::try_parse_from(["malvin", "--creative=0", "--doc"]).expect("parse");
        assert_eq!(zero.router.creative, Some(0.0));
        assert!(!zero.router.sample_creative_this_iteration());
    }

    #[test]
    fn creative_probability_rejects_out_of_range() {
        assert!(super::parse_creative_probability("1.1").is_err());
        assert!(super::parse_creative_probability("-0.1").is_err());
        assert!(super::parse_creative_probability("nope").is_err());
        assert_eq!(super::parse_creative_probability("0.5").ok(), Some(0.5));
    }

    #[test]
    fn iml_flag_defaults_off_and_parses() {
        use clap::Parser;
        let off = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert!(!off.shared.iml);

        let on = crate::cli::Cli::try_parse_from(["malvin", "--iml", "--doc"]).expect("parse");
        assert!(on.shared.iml);
    }

    #[test]
    fn help_lists_iml_as_infinite_meta_loop() {
        use clap::CommandFactory;
        let help = crate::cli::Cli::command().render_help().to_string();
        assert!(help.contains("--iml"), "help={help}");
        assert!(
            help.contains("the Infinite Meta-Loop"),
            "help must label --iml as the Infinite Meta-Loop; got {help}"
        );
    }
}
