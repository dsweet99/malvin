use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// How many separate kpop agent runs to perform (one Popper session per iteration).
    #[arg(long, default_value_t = 1)]
    pub max_loops: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    /// Existing `.md` path, literal text, or KPOP log id for lookup.
    #[arg(value_name = "REQUEST", num_args = 0..)]
    pub requests: Vec<String>,
}

impl KpopArgs {
    pub(crate) fn first_request(&self) -> Option<&String> {
        self.requests.first()
    }

    pub(crate) fn is_lookup(&self) -> bool {
        self.requests.len() == 1
            && crate::cli::bug_id_lookup_kpop::is_kpop_lookup_request(
                self.first_request().map(String::as_str),
            )
    }

    pub(crate) fn with_request(&self, request: String) -> Self {
        Self {
            max_loops: self.max_loops,
            tenacious: self.tenacious,
            requests: vec![request],
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;

    #[test]
    fn kpop_max_loops_help_excludes_mpc_plan_early_exit() {
        let cmd = Cli::command();
        let kpop = cmd
            .get_subcommands()
            .find(|sub| sub.get_name() == "kpop")
            .expect("kpop subcommand");
        let arg = kpop
            .get_arguments()
            .find(|a| a.get_id() == "max_loops")
            .expect("max_loops flag");
        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .expect("max_loops help")
            .to_string()
            .to_lowercase();
        assert!(
            !help.contains("mpc plan"),
            "max_loops help must not mention mpc plan: {help}"
        );
        assert!(
            !help.contains("done"),
            "max_loops help must not mention DONE early exit: {help}"
        );
        assert!(
            help.contains("run") || help.contains("iteration"),
            "max_loops help should describe iteration budget: {help}"
        );
    }
}
