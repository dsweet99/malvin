use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct BugArgs {
    /// After discovery, write regression test and fix
    #[arg(long, default_value_t = false, conflicts_with = "bug_id")]
    pub fix: bool,
    /// Total `KPop` hypothesis steps before stopping (same as `malvin kpop`).
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// Skip learning after `KPop`.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Skip workspace quality gates before the post-KPOP coder session.
    #[arg(long, default_value_t = false)]
    pub skip_pre_checks: bool,
    /// Fix an existing bug by id (skip discovery).
    #[arg(value_name = "BUG_ID", conflicts_with = "fix")]
    pub bug_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// Total `KPop` hypothesis steps (## Step headings in the exp log) before stopping.
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../request.md`.
    pub request: Option<String>,
}
