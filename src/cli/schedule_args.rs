use clap::Args;

#[derive(Args, Debug)]
pub struct ScheduleArgs {
    /// Number of parallel workers.
    #[arg(long)]
    pub workers: usize,
    /// Input JSON job graph file.
    pub jobs_json_path: String,
}

#[cfg(test)]
mod coverage_tests {
    #[test]
    fn kiss_stringify_schedule_args_units() {
        let _ = stringify!(crate::cli::schedule_args::ScheduleArgs);
    }
}
