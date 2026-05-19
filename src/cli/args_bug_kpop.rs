use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct BugArgs {
    /// Total KPOP + MBC2 hypothesis steps before stopping (same as `malvin kpop`).
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// MBC2 interleave density (same as `malvin kpop`).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learning after KPOP.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Skip workspace quality gates before the post-KPOP coder session.
    #[arg(long, default_value_t = false)]
    pub skip_pre_checks: bool,
}

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// Total KPOP + MBC2 hypothesis steps (## Step headings in the exp log) before stopping.
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// Drives mean KPOP block size and MBC2 interleave; higher = more frequent MBC2 turns and smaller KPOP blocks. Non-finite or ≤ 0 disables MBC2 turns (pure multiturn KPOP).
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    /// Skip learning.
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    /// Request or `@file` → `_malvin/.../request.md`.
    pub request: String,
}
