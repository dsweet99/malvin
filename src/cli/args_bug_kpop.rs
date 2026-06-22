use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// How many times to run the kpop agent (stops early when the exp log contains `## KPOP_SOLVED`).
    #[arg(long, default_value_t = 1)]
    pub max_loops: usize,
    /// Total `KPop` hypothesis steps (## Step headings in the exp log) per agent run.
    #[arg(long, default_value_t = 5)]
    pub max_hypotheses: usize,
    /// Expand to `--max-acp-retries=9999` and `--max-loops=9999`.
    #[arg(long, default_value_t = crate::cli::loop_opts::DEFAULT_TENACIOUS)]
    pub tenacious: bool,
    /// Existing `.md` path or literal text → `.malvin/logs/.../request.md`.
    pub request: Option<String>,
}

#[cfg(test)]
#[path = "args_bug_kpop_test.rs"]
mod args_bug_kpop_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<KpopArgs> = None;
    }
}
