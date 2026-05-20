use std::path::Path;

use super::types::RepoGateOutput;

fn append_quality_gates_log_text(run_dir: &Path, text: &str) -> std::io::Result<()> {
    let path = run_dir.join(crate::artifacts::QUALITY_GATES_LOG);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    std::io::Write::write_all(&mut f, text.as_bytes())
}

fn append_quality_gates_log_line(run_dir: &Path, who: &str, line: &str) -> std::io::Result<()> {
    use crate::output::format_line;
    append_quality_gates_log_text(run_dir, &format!("{}\n", format_line(who, line)))
}

fn try_append_log_line(run_log_dir: Option<&Path>, who: &str, line: &str) {
    if let Some(dir) = run_log_dir {
        if let Err(e) = append_quality_gates_log_line(dir, who, line) {
            use crate::output::print_log_warning;
            print_log_warning(&format!("failed to write quality gates log: {e}"));
        }
    }
}

pub(crate) fn try_append_command_output(
    run_log_dir: Option<&Path>,
    command_line: &str,
    cmd_output: &std::process::Output,
) {
    if let Some(dir) = run_log_dir {
        if let Err(e) = append_quality_gates_command_output(dir, command_line, cmd_output) {
            emit_repo_gate_warning(
                &format!("failed to write quality gates log for `{command_line}`: {e}"),
                Some(dir),
            );
        }
    }
}

pub(crate) fn emit_repo_gate_warning(line: &str, run_log_dir: Option<&Path>) {
    use crate::output::{WARNING_WHO, print_log_warning};
    print_log_warning(line);
    try_append_log_line(run_log_dir, WARNING_WHO, line);
}

pub(crate) fn emit_repo_gate_line(output: RepoGateOutput, line: &str, run_log_dir: Option<&Path>) {
    use crate::output::{MALVIN_WHO, print_stderr_line, print_stdout_line};
    match output {
        RepoGateOutput::Tagged => {
            print_stdout_line(MALVIN_WHO, line);
            try_append_log_line(run_log_dir, MALVIN_WHO, line);
        }
        RepoGateOutput::Stderr => {
            print_stderr_line(MALVIN_WHO, line);
            try_append_log_line(run_log_dir, MALVIN_WHO, line);
        }
    }
}

pub(crate) fn append_quality_gates_command_output(
    run_dir: &Path,
    command_line: &str,
    output: &std::process::Output,
) -> std::io::Result<()> {
    use crate::output::{MALVIN_WHO, format_line};
    let exit = output
        .status
        .code()
        .map_or_else(|| "signal".to_string(), |code| code.to_string());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    append_quality_gates_log_text(
        run_dir,
        &format!(
            "{}\nexit code: {exit}\n[stdout]\n{stdout}\n[stderr]\n{stderr}\n",
            format_line(MALVIN_WHO, &format!("Finished `{command_line}`"))
        ),
    )
}

#[cfg(test)]
mod gate_log_tests {
    #[test]
    fn append_quality_gates_log_writes_run_dir_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        super::append_quality_gates_log_text(tmp.path(), "plain\n").expect("text");
        super::append_quality_gates_log_line(tmp.path(), "who", "line").expect("line");
        let path = tmp.path().join(crate::artifacts::QUALITY_GATES_LOG);
        let content = std::fs::read_to_string(path).expect("read log");
        assert!(content.contains("plain"));
        assert!(content.contains("line"));
        super::emit_repo_gate_warning("warn", Some(tmp.path()));
        super::emit_repo_gate_line(super::RepoGateOutput::Tagged, "ok", Some(tmp.path()));
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"out".to_vec(),
            stderr: b"err".to_vec(),
        };
        super::append_quality_gates_command_output(tmp.path(), "kiss", &output).expect("cmd out");
    }

    #[test]
    fn emit_repo_gate_warning_survives_blocked_log_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(tmp.path().join(crate::artifacts::QUALITY_GATES_LOG)).expect("block log");
        super::emit_repo_gate_warning("warn", Some(tmp.path()));
    }

    #[test]
    fn try_append_command_output_survives_blocked_log_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir(tmp.path().join(crate::artifacts::QUALITY_GATES_LOG)).expect("block log");
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        super::try_append_command_output(Some(tmp.path()), "kiss check", &output);
    }
}
