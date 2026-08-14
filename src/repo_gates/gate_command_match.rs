use std::path::Path;

use super::load_malvin_checks;

fn gate_line_matches_command(gate_line: &str, cmd: &str) -> bool {
    let gate = gate_line.trim();
    if gate.is_empty() {
        return false;
    }
    let cmd_lower = cmd.trim().to_ascii_lowercase();
    let gate_lower = gate.to_ascii_lowercase();
    cmd_lower.contains(&gate_lower) || gate_lower.contains(&cmd_lower)
}

#[must_use]
pub fn command_matches_malvin_checks_gate(cmd: &str, work_dir: &Path) -> bool {
    let checks_path = crate::resolve_malvin_checks_path(work_dir);
    if !checks_path.is_file() {
        return false;
    }
    let Ok(lines) = load_malvin_checks(&checks_path) else {
        return false;
    };
    lines
        .iter()
        .any(|line| gate_line_matches_command(line, cmd))
}
