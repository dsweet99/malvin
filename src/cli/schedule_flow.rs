use std::fs::read_to_string;

use malvin::schedule;

use crate::cli::schedule_args::ScheduleArgs;

pub fn run_schedule(args: &ScheduleArgs) -> Result<(), String> {
    let jobs_json = read_to_string(&args.jobs_json_path)
        .map_err(|e| format!("ERR:failed to read jobs file {}: {e}", args.jobs_json_path))?;
    let scheduled = schedule::run_schedule_json(&jobs_json, args.workers)?;
    let serialized = schedule::render_schedule_json(&scheduled);
    println!("{serialized}");
    Ok(())
}
