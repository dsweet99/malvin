use clap::Args;

#[derive(Args, Debug)]
pub struct ScheduleArgs {
    /// JSON file containing job definitions.
    pub jobs_path: String,
    #[arg(short, long, default_value_t = 1)]
    pub workers: usize,
}

