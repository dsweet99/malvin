//! **`KPopLogProtocol`** — parsed markers in `exp_log_*.md` (see `src/kpop_engine/`).
//!
//! Agents write `exp_log_*.md` under `_kpop/` with markdown section markers malvin
//! interprets for budget checks and early exit. Prompt sources: `default_prompts/kpop_common.md`,
//! `default_prompts/kpop_block.md`.

use std::path::Path;

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
    pub fn read(path: &Path) -> Result<Self, String> {
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
    pub fn hypothesis_step_count(&self) -> usize {
        self.kpop_step_count() + self.mbc2_step_count()
    }

    #[must_use]
    pub fn declares_kpop_solved(&self) -> bool {
        self.kpop_solved_marker_count() > 0
    }

    #[must_use]
    pub fn kpop_solved_marker_count(&self) -> usize {
        self.text
            .lines()
            .filter(|line| marker_line_is_exact("## KPOP_SOLVED", line))
            .count()
    }

    #[must_use]
    pub fn mpc_done_marker_count(&self) -> usize {
        self.text
            .lines()
            .filter(|line| marker_line_is_exact("## MPC_DONE", line))
            .count()
    }

    /// Fail when hypothesis steps exceed `max`.
    ///
    /// # Errors
    ///
    /// Returns the shared budget-exceeded message used by gate and multiturn flows.
    pub fn check_hypothesis_budget(&self, max: usize) -> Result<(), String> {
        let count = self.hypothesis_step_count();
        if count > max {
            return Err(format!(
                "experiment log counts {count} hypothesis steps, exceeding --max-hypotheses ({max})"
            ));
        }
        Ok(())
    }
}

fn marker_line_is_exact(marker: &str, line: &str) -> bool {
    let t = line.trim_start();
    t.strip_prefix(marker)
        .is_some_and(|rest| rest.trim().is_empty())
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
