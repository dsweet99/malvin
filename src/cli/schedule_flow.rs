use std::fs;

use malvin::schedule::{render_schedule_json, run_schedule_json};

use super::schedule_args::ScheduleArgs;

pub fn run_schedule(args: &ScheduleArgs) -> Result<(), String> {
    let input =
        fs::read_to_string(&args.jobs_path).map_err(|e| format!("ERR:failed to read input: {e}"))?;
    let scheduled = run_schedule_json(&input, args.workers)?;
    print!("{}", render_schedule_json(&scheduled));
    Ok(())
}

