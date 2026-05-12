use malvin::output::{MALVIN_WHO, format_line, print_stdout_line};
use std::path::Path;

use super::types::RepoGateOutput;

fn append_quality_checks_log_line(run_dir: &Path, line: &str) -> std::io::Result<()> {
    let path = run_dir.join("quality_checks.log");
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let s = format!("{}\n", format_line(MALVIN_WHO, line));
    std::io::Write::write_all(&mut f, s.as_bytes())
}

pub fn emit_repo_gate_line(output: RepoGateOutput, line: &str, run_log_dir: Option<&Path>) {
    match output {
        RepoGateOutput::Tagged => print_stdout_line(MALVIN_WHO, line),
        RepoGateOutput::Stderr => eprintln!("{line}"),
    }
    if let Some(dir) = run_log_dir {
        let _ = append_quality_checks_log_line(dir, line);
    }
}

#[cfg(test)]
mod kiss_stringify_emit {
    #[test]
    fn kiss_stringify_repo_checks_emit_append_log() {
        let _ = stringify!(super::append_quality_checks_log_line);
    }
}
