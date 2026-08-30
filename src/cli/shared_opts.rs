pub use crate::config::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::parser::ValueSource;
use clap::{ArgMatches, Args};
use rand::Rng;

use crate::model_id::{parse_model_id, ParsedModel};

const QUIET_HELPTEXT: &str =
    "Print only `__MALVIN_DM_START__`/`END` bodies on stdout (default router; not `-b`)";

const CREATIVE_HELPTEXT: &str =
    "Be (more) creative; optional probability in [0,1] (default 1.0 when set)";

/// Parse `--creative[=PROB]` values in `[0.0, 1.0]`.
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

#[derive(Args, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SharedOpts {
    /// Suppress all stdout
    #[arg(short = 'b', long, default_value_t = false)]
    pub background: bool,
    /// Model id (`cursor:`, `pi:`, or `codex:`)
    #[arg(
        long,
        default_value = DEFAULT_CLI_MODEL,
        value_parser = parse_model_id
    )]
    pub model: ParsedModel,
    /// Do not auto-approve tool calls (unsupported on `cursor:`, `pi:`, and `codex:`; fails fast)
    #[arg(long, default_value_t = false)]
    pub no_force: bool,
    /// Do not expand gate-loop budgets to tenacious limits (tenacious on by default)
    #[arg(long = "no-tenacious", default_value_t = false)]
    pub no_tenacious: bool,
    /// Run workspace quality gates; treat failures as loop or exit criteria
    #[arg(short = 'g', long, default_value_t = false)]
    pub gates: bool,
    /// Print only `__MALVIN_DM_START__`/`END` bodies on stdout (default router; not `-b`)
    #[arg(
        short = 'q',
        long,
        default_value_t = false,
        help = QUIET_HELPTEXT
    )]
    pub quiet: bool,
    /// Log full outgoing agent prompts to stdout and `prompts.log`
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Max agent retries per spawn or gate iteration
    #[arg(long = "max-acp-retries", default_value_t = DEFAULT_MAX_ACP_RETRIES)]
    pub max_acp_retries: u32,
    /// Print built-in documentation and exit
    #[arg(long, global = true, default_value_t = false)]
    pub doc: bool,
    /// Allow the agent to run `git commit`
    #[arg(long, default_value_t = false)]
    pub git: bool,
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
    /// Turn off `KPop`
    #[arg(long = "no-kpop", default_value_t = false, hide = true)]
    pub no_kpop: bool,
}

/// Copy agent flags set on a subcommand over root `SharedOpts` (non-global shared opts).
pub(crate) fn overlay_shared_opts_from_subcommand(
    base: &mut SharedOpts,
    sub: &SharedOpts,
    matches: &ArgMatches,
    subcommand: &str,
) {
    let Some(sub_m) = matches.subcommand().filter(|(n, _)| *n == subcommand).map(|(_, m)| m)
    else {
        return;
    };
    let on_cli = |id: &str| {
        sub_m
            .value_source(id)
            .is_some_and(|source| source == ValueSource::CommandLine)
    };
    if on_cli("model") {
        base.model = sub.model.clone();
    }
    if on_cli("max_acp_retries") {
        base.max_acp_retries = sub.max_acp_retries;
    }
    overlay_shared_bool_fields(base, sub, &on_cli);
}

fn overlay_shared_bool_fields(
    base: &mut SharedOpts,
    sub: &SharedOpts,
    on_cli: &impl Fn(&str) -> bool,
) {
    overlay_shared_bool_if(on_cli, "background", &mut base.background, sub.background);
    overlay_shared_bool_if(on_cli, "no_force", &mut base.no_force, sub.no_force);
    overlay_shared_bool_if(on_cli, "no_tenacious", &mut base.no_tenacious, sub.no_tenacious);
    overlay_shared_bool_if(on_cli, "gates", &mut base.gates, sub.gates);
    overlay_shared_bool_if(on_cli, "quiet", &mut base.quiet, sub.quiet);
    overlay_shared_bool_if(on_cli, "verbose", &mut base.verbose, sub.verbose);
    overlay_shared_bool_if(on_cli, "git", &mut base.git, sub.git);
    if on_cli("creative") {
        base.creative = sub.creative;
    }
    overlay_shared_bool_if(on_cli, "no_kpop", &mut base.no_kpop, sub.no_kpop);
}

fn overlay_shared_bool_if(
    on_cli: &impl Fn(&str) -> bool,
    id: &str,
    dst: &mut bool,
    src: bool,
) {
    if on_cli(id) {
        *dst = src;
    }
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

    /// Sample whether this outer router iteration applies creative turns.
    ///
    /// When `--creative` is unset, returns false. When set, both the post-kpop
    /// `mbc2.md` turn and `router_b_creative.md` (vs `router_b.md`) apply together
    /// with the configured probability (default 1.0).
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
            background: false,
            model: parse_model_id(crate::config::DEFAULT_CLI_MODEL).expect("default model"),
            no_force: true,
            no_tenacious: false,
            gates: false,
            quiet: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
            git: false,
            creative: None,
            no_kpop: false,
        }
    }
}

#[cfg(test)]
mod overlay_tests {
    use super::overlay_shared_opts_from_subcommand;
    use crate::cli::{Cli, Commands, SharedOpts};
    use clap::{CommandFactory, FromArgMatches};

    #[test]
    fn overlay_prefers_write_subcommand_model() {
        let matches = Cli::command().get_matches_from([
            "malvin",
            "write",
            "--model",
            "cursor:sonnet-4",
            "topic",
        ]);
        let cli = Cli::from_arg_matches(&matches).expect("from matches");
        let mut shared = SharedOpts::test_defaults();
        let Commands::Write(write) = cli.command.expect("write") else {
            panic!("expected write");
        };
        overlay_shared_opts_from_subcommand(&mut shared, &write.shared, &matches, "write");
        assert_eq!(shared.model.canonical(), "cursor:sonnet-4");
    }

    #[test]
    fn overlay_keeps_root_model_when_write_omits_flag() {
        let matches = Cli::command().get_matches_from([
            "malvin",
            "--model",
            "cursor:composer-2",
            "write",
            "topic",
        ]);
        let cli = Cli::from_arg_matches(&matches).expect("from matches");
        let mut shared = cli.shared.clone();
        let Commands::Write(write) = cli.command.expect("write") else {
            panic!("expected write");
        };
        overlay_shared_opts_from_subcommand(&mut shared, &write.shared, &matches, "write");
        assert_eq!(shared.model.canonical(), "cursor:composer-2");
    }

    #[test]
    fn creative_flag_defaults_off_and_accepts_probability() {
        use clap::Parser;
        let off = crate::cli::Cli::try_parse_from(["malvin", "--doc"]).expect("parse");
        assert!(off.shared.creative.is_none());
        assert!(!off.shared.sample_creative_this_iteration());

        let on = crate::cli::Cli::try_parse_from(["malvin", "--creative", "--doc"]).expect("parse");
        assert_eq!(on.shared.creative, Some(1.0));
        assert!(on.shared.sample_creative_this_iteration());

        let p = crate::cli::Cli::try_parse_from(["malvin", "--creative=0.6", "--doc"]).expect("parse");
        assert_eq!(p.shared.creative, Some(0.6));

        let zero =
            crate::cli::Cli::try_parse_from(["malvin", "--creative=0", "--doc"]).expect("parse");
        assert_eq!(zero.shared.creative, Some(0.0));
        assert!(!zero.shared.sample_creative_this_iteration());
    }

    #[test]
    fn creative_probability_rejects_out_of_range() {
        assert!(super::parse_creative_probability("1.1").is_err());
        assert!(super::parse_creative_probability("-0.1").is_err());
        assert!(super::parse_creative_probability("nope").is_err());
        assert_eq!(super::parse_creative_probability("0.5").ok(), Some(0.5));
    }

    #[test]
    fn overlay_prefers_write_subcommand_creative() {
        let matches = Cli::command().get_matches_from([
            "malvin",
            "write",
            "--creative=0.4",
            "topic",
        ]);
        let cli = Cli::from_arg_matches(&matches).expect("from matches");
        let mut shared = SharedOpts::test_defaults();
        let Commands::Write(write) = cli.command.expect("write") else {
            panic!("expected write");
        };
        overlay_shared_opts_from_subcommand(&mut shared, &write.shared, &matches, "write");
        assert_eq!(shared.creative, Some(0.4));
    }
}
