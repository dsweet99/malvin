use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct BugArgs {
    /// After discovery, write regression test and fix
    #[arg(long, default_value_t = false, conflicts_with = "bug_id")]
    pub fix: bool,
    /// Total `KPop` + MBC2 hypothesis steps before stopping (same as `malvin kpop`).
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// MBC2 interleave density (same as `malvin kpop`).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learning after `KPop`.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Fix an existing bug by id (skip discovery).
    #[arg(value_name = "BUG_ID", conflicts_with = "fix")]
    pub bug_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// Total `KPop` + MBC2 hypothesis steps (## Step headings in the exp log) before stopping.
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// Drives mean `KPop` block size and MBC2 interleave; higher = more frequent MBC2 turns and smaller `KPop` blocks. Non-finite or ≤ 0 disables MBC2 turns (pure multiturn `KPop`).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../request.md`.
    pub request: Option<String>,
}
