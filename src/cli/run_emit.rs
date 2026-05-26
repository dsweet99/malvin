use std::io::Write;
use std::path::Path;

use crate::artifacts::{RunArtifacts, startup_request_tag_label};
use crate::format_logs_dir;
use crate::mem_limit_config::format_host_resources_line;
use crate::output::{MALVIN_WHO, format_line, print_stdout_line, print_stdout_text};

pub fn emit_command_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    crate::init_from_env();
    let cmd = crate::command_line().expect("init_from_env populates argv via OnceLock");
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

pub fn emit_host_resources_line(run_dir: &Path, echo_stdout: bool) -> Result<(), String> {
    let line = format_host_resources_line();
    if echo_stdout {
        print_stdout_line(MALVIN_WHO, &line);
    }
    let log_path = run_dir.join("command.log");
    let formatted = format!("{}\n", format_line(MALVIN_WHO, &line));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|e| format!("command.log: {e}"))?;
    file.write_all(formatted.as_bytes())
        .map_err(|e| format!("command.log: {e}"))?;
    Ok(())
}

pub struct RunStartupEmitOpts {
    pub tee_stdout: bool,
    pub host_resources: bool,
}

pub fn emit_run_startup_sequence(
    artifacts: &RunArtifacts,
    opts: RunStartupEmitOpts,
    cli_request: &str,
) -> Result<(), String> {
    emit_command_line(&artifacts.run_dir, opts.tee_stdout)?;
    if opts.host_resources {
        emit_host_resources_line(&artifacts.run_dir, opts.tee_stdout)?;
    }
    let tag = startup_request_tag_label(cli_request);
    echo_primary_to_stdout(&artifacts.plan_path, opts.tee_stdout, &tag)?;
    print_stdout_line(
        MALVIN_WHO,
        &format!("Logs: {}", format_logs_dir(&artifacts.run_dir)?),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{emit_host_resources_line, emit_run_startup_sequence, RunStartupEmitOpts};

    #[test]
    fn emit_host_resources_line_appends_to_command_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join("run");
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(run_dir.join("command.log"), "existing\n").expect("seed");
        emit_host_resources_line(&run_dir, false).expect("emit");
        let text = std::fs::read_to_string(run_dir.join("command.log")).expect("read");
        assert!(text.contains("existing"));
        assert!(text.contains("Memory:"));
        assert!(text.contains("CPUs:"));
    }

    #[test]
    fn emit_run_startup_sequence_includes_host_resources_when_requested() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_run_artifacts_from_text("hi", Some(tmp.path())).expect("art");
        emit_run_startup_sequence(
            &artifacts,
            RunStartupEmitOpts {
                tee_stdout: false,
                host_resources: true,
            },
            "hi",
        )
        .expect("startup");
        let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
        assert!(log.contains("Command:"));
        assert!(log.contains("Memory:"));
        assert!(log.contains("CPUs:"));
    }

    #[test]
    fn emit_run_startup_sequence_omits_host_resources_when_disabled() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts =
            crate::artifacts::create_run_artifacts_from_text("init", Some(tmp.path())).expect("art");
        emit_run_startup_sequence(
            &artifacts,
            RunStartupEmitOpts {
                tee_stdout: false,
                host_resources: false,
            },
            "init",
        )
        .expect("startup");
        let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
        assert!(log.contains("Command:"));
        assert!(!log.contains("Memory:"));
    }
}
