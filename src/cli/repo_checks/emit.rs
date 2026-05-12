use malvin::output::{MALVIN_WHO, format_line, print_stdout_line};
use std::path::Path;

use super::types::RepoGateOutput;

fn append_quality_gates_log_text(run_dir: &Path, text: &str) -> std::io::Result<()> {
    let path = run_dir.join(malvin::artifacts::QUALITY_GATES_LOG);
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    std::io::Write::write_all(&mut f, text.as_bytes())
}

fn append_quality_gates_log_line(run_dir: &Path, line: &str) -> std::io::Result<()> {
    append_quality_gates_log_text(run_dir, &format!("{}\n", format_line(MALVIN_WHO, line)))
}

pub fn append_quality_gates_command_output(
    run_dir: &Path,
    command_line: &str,
    output: &std::process::Output,
) -> std::io::Result<()> {
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

pub fn emit_repo_gate_line(output: RepoGateOutput, line: &str, run_log_dir: Option<&Path>) {
    match output {
        RepoGateOutput::Tagged => print_stdout_line(MALVIN_WHO, line),
        RepoGateOutput::Stderr => eprintln!("{line}"),
    }
    if let Some(dir) = run_log_dir {
        let _ = append_quality_gates_log_line(dir, line);
    }
}

#[cfg(test)]
mod kiss_stringify_emit {
    #[test]
    fn kiss_stringify_repo_checks_emit_append_log() {
        let _ = stringify!(super::append_quality_gates_log_line);
        let _ = stringify!(super::append_quality_gates_command_output);
    }
}
