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
