use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    /// Total `KPop` hypothesis steps (## Step headings in the exp log) before stopping.
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    /// Request or `@file` → `.malvin/logs/.../request.md`.
    pub request: Option<String>,
}
