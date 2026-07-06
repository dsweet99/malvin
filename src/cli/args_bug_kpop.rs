use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// How many separate kpop agent runs to perform (one Popper session per iteration).
    #[arg(long, default_value_t = 1)]
    pub max_loops: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    /// Existing `.md` path or literal text → `.malvin/logs/.../request.md`.
    pub request: Option<String>,
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
