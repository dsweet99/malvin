//! Coder-prompt phase machine (see `concepts.md` §6).
//!
//! Inside each `run_coder_prompt`, the mini loop moves through `Investigate`, `WindDown`, and
//! `Terminal` phases. This module names the phase enum for documentation and typing; transition
//! logic stays in `loop_inner_phases` and related mini modules.

/// Phase of the mini inner loop for one coder prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniPhase {
    Investigate,
    WindDown,
    Terminal,
}

impl MiniPhase {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Investigate => "investigate",
            Self::WindDown => "wind_down",
            Self::Terminal => "terminal",
        }
    }
}

#[cfg(test)]
#[path = "coder_prompt_phase_tests.rs"]
mod coder_prompt_phase_tests;
