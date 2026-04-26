use clap::Args;

#[derive(Args, Debug)]
pub struct ScheduleArgs {
    /// Number of parallel workers.
    #[arg(long)]
    pub workers: usize,
    /// Input JSON job graph file.
    pub jobs_json_path: String,
}
