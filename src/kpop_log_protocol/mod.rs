//! **`KPopLogProtocol`** — parsed step headings in `exp_log_*.md` (see `src/kpop_engine/`).
//!
//! Agents write `exp_log_*.md` under `_kpop/` with markdown section markers malvin
//! interprets for observability (step counts). Prompt source: `default_prompts/kpop_common.md`.

/// Parsed marker kind on a `## Step K — …` heading line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepHeadingKind {
    KPop,
    Mbc2,
}

/// A parsed step heading (index and kind only; hypothesis blocks are not structured).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepHeading {
    pub index: usize,
    pub kind: StepHeadingKind,
}

/// Parsed experiment log text with query helpers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExperimentLog {
    text: String,
}

impl ExperimentLog {
    /// Read and parse an experiment log file.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the file cannot be read.
    pub fn read(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read exp log {}: {e}", path.display()))?;
        Ok(Self::from_text(text))
    }

    #[must_use]
    pub fn from_text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn kpop_step_count(&self) -> usize {
        self.text
            .lines()
            .filter(|line| step_kind(line) == Some(StepHeadingKind::KPop))
            .count()
    }

    #[must_use]
    pub fn mbc2_step_count(&self) -> usize {
        self.text
            .lines()
            .filter(|line| step_kind(line) == Some(StepHeadingKind::Mbc2))
            .count()
    }

    #[must_use]
    pub fn kpop_solved_marker_count(&self) -> usize {
        self.text.lines().filter(|line| is_kpop_solved_line(line)).count()
    }

    #[must_use]
    pub fn declares_solved(&self) -> bool {
        self.kpop_solved_marker_count() > 0
    }

}

fn is_kpop_solved_line(line: &str) -> bool {
    let t = line.trim_start();
    let Some(rest) = t.strip_prefix("## KPOP_SOLVED") else {
        return false;
    };
    rest.is_empty()
        || rest.starts_with(|c: char| c.is_whitespace() || matches!(c, '-' | '—' | '–' | ':'))
}

fn is_kpop_step_label(tail: &str) -> bool {
    if tail.len() < 4 || !tail[..4].eq_ignore_ascii_case("kpop") {
        return false;
    }
    tail.len() == 4 || !tail.as_bytes()[4].is_ascii_alphanumeric()
}

fn step_kind(line: &str) -> Option<StepHeadingKind> {
    let t = line.trim_start();
    let rest = t.strip_prefix("## Step ")?;
    let tail = [" — ", " – ", " - "]
        .iter()
        .find_map(|sep| rest.split_once(sep).map(|(_, t)| t))?;
    let tail = tail.trim_start();
    if is_kpop_step_label(tail) {
        return Some(StepHeadingKind::KPop);
    }
    if tail.starts_with("MBC2") {
        return Some(StepHeadingKind::Mbc2);
    }
    None
}

#[cfg(test)]
#[path = "log_protocol_tests.rs"]
mod log_protocol_tests;
