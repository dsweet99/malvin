#[derive(clap::Args, Debug, Clone)]
pub struct BugArgs {
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    #[arg(long, default_value_t = false)]
    pub skip_pre_checks: bool,
}

#[derive(Args, Debug, Clone)]
pub struct KpopArgs {
    #[arg(long, default_value_t = 10, alias = "max-loops")]
    pub max_hypotheses: usize,
    #[arg(long, default_value_t = 0.10)]
    pub p_creative: f64,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
    pub request: String,
}
