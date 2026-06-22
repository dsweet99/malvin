use std::path::Path;

/// Reads the experiment log at `path` into a string.
///
/// # Errors
///
/// Returns `Err` when the file cannot be read.
pub fn read_exp_log_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read exp log {}: {e}", path.display()))
}

fn is_kpop_step_label(tail: &str) -> bool {
    if tail.len() < 4 || !tail[..4].eq_ignore_ascii_case("kpop") {
        return false;
    }
    tail.len() == 4 || !tail.as_bytes()[4].is_ascii_alphanumeric()
}

fn step_kind(line: &str) -> Option<&'static str> {
    let t = line.trim_start();
    let rest = t.strip_prefix("## Step ")?;
    let tail = [" — ", " – ", " - "]
        .iter()
        .find_map(|sep| rest.split_once(sep).map(|(_, t)| t))?;
    let tail = tail.trim_start();
    if is_kpop_step_label(tail) {
        return Some("KPop");
    }
    if tail.starts_with("MBC2") {
        return Some("MBC2");
    }
    None
}

#[must_use]
pub fn count_kpop_entries(text: &str) -> usize {
    text.lines()
        .filter(|line| step_kind(line) == Some("KPop"))
        .count()
}

#[must_use]
pub fn count_mbc2_entries(text: &str) -> usize {
    text.lines()
        .filter(|line| step_kind(line) == Some("MBC2"))
        .count()
}

#[must_use]
pub fn hypotheses_emitted(text: &str) -> usize {
    count_kpop_entries(text) + count_mbc2_entries(text)
}

#[must_use]
pub fn count_kpop_solved_markers(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let t = line.trim_start();
            t.strip_prefix("## KPOP_SOLVED")
                .is_some_and(|rest| rest.trim().is_empty())
        })
        .count()
}

#[must_use]
pub fn agent_declared_success(text: &str) -> bool {
    count_kpop_solved_markers(text) > 0
}

#[cfg(test)]
mod tests {
    use super::{is_kpop_step_label, step_kind};

    #[test]
    fn step_kind_classifies_kpop_mbc2_and_rejects_kpopulation() {
        assert_eq!(step_kind("## Step 1 — KPop x"), Some("KPop"));
        assert_eq!(step_kind("## Step 2 — MBC2 y"), Some("MBC2"));
        assert_eq!(step_kind("## Step 3 — kpopulation x"), None);
    }

    #[test]
    fn is_kpop_step_label_accepts_kpop_prefix_only() {
        assert!(is_kpop_step_label("KPop"));
        assert!(is_kpop_step_label("kpop"));
        assert!(!is_kpop_step_label("kpopulation"));
        assert!(!is_kpop_step_label("foo"));
    }
}
#[cfg(test)]
#[path = "counters_test.rs"]
mod counters_test;
