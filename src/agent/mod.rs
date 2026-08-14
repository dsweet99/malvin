
use crate::acp::CoderPromptOptions;

#[derive(Debug, Clone, Copy, Default)]
pub struct PromptOptions {
    pub single_attempt: bool,
}

impl PromptOptions {
    #[must_use]
    pub const fn from_coder(opts: &CoderPromptOptions<'_>) -> Self {
        Self {
            single_attempt: opts.single_attempt,
        }
    }
}

#[cfg(test)]
#[path = "kiss_coverage.rs"]
mod kiss_coverage;
