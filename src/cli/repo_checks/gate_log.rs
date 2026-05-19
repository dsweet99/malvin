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

pub(crate) fn emit_repo_gate_warning(line: &str, run_log_dir: Option<&Path>) {
    use crate::output::{WARNING_WHO, print_log_warning};
    print_log_warning(line);
    if let Some(dir) = run_log_dir {
        let _ = append_quality_gates_log_line(dir, WARNING_WHO, line);
    }
}

pub(crate) fn emit_repo_gate_line(output: RepoGateOutput, line: &str, run_log_dir: Option<&Path>) {
    use crate::output::{MALVIN_WHO, print_stderr_line, print_stdout_line};
    match output {
        RepoGateOutput::Tagged => {
            print_stdout_line(MALVIN_WHO, line);
            if let Some(dir) = run_log_dir {
                let _ = append_quality_gates_log_line(dir, MALVIN_WHO, line);
            }
        }
        RepoGateOutput::Stderr => {
            print_stderr_line(MALVIN_WHO, line);
            if let Some(dir) = run_log_dir {
                let _ = append_quality_gates_log_line(dir, MALVIN_WHO, line);
            }
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
mod kiss_stringify_gate_log {
    #[test]
    fn kiss_stringify_gate_log_units() {
        let _ = stringify!(super::emit_repo_gate_warning);
        let _ = stringify!(super::emit_repo_gate_line);
        let _ = stringify!(super::append_quality_gates_command_output);
    }
}
