use std::path::Path;

use malvin::artifacts::{RunArtifacts, startup_request_tag_label};
use malvin::format_logs_dir;
use malvin::output::{MALVIN_WHO, format_line, print_stdout_line, print_stdout_text};

pub fn emit_command_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    malvin::init_from_env();
    let cmd =
        malvin::command_line().expect("init_from_env populates argv via OnceLock");
    let line = format!("Command: {cmd}");
    if echo_stdout {
        print_stdout_line(MALVIN_WHO, &line);
    }
    let log_path = run_dir.join("command.log");
    std::fs::write(&log_path, format!("{}\n", format_line(MALVIN_WHO, &line)))
        .map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

pub fn echo_primary_to_stdout(
    plan_path: &Path,
    echo_plain: bool,
    startup_tag_label: &str,
) -> Result<(), String> {
    if !echo_plain {
        return Ok(());
    }
    let plan_text = std::fs::read_to_string(plan_path).map_err(|e| e.to_string())?;
    print_stdout_text(startup_tag_label, &plan_text);
    Ok(())
}

pub fn emit_run_startup_sequence(
    artifacts: &RunArtifacts,
    tee_startup_stdout: bool,
    cli_request: &str,
) -> Result<(), String> {
    emit_command_line(&artifacts.run_dir, tee_startup_stdout)?;
    let tag = startup_request_tag_label(cli_request);
    echo_primary_to_stdout(&artifacts.plan_path, tee_startup_stdout, &tag)?;
    print_stdout_line(
        MALVIN_WHO,
        &format!("Logs: {}", format_logs_dir(&artifacts.run_dir)?),
    );
    Ok(())
}
