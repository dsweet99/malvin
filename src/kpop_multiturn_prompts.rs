use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};
use crate::kpop_turn_prompts::KpopTurnPrompts;

#[cfg(test)]
#[derive(Debug)]
pub struct SmokeKpopBuilder;

#[derive(Debug)]
pub enum KpopMultiturnPrompts<'a> {
    Turn(KpopTurnPrompts<'a>),
    StubMt(MtStubPrompts),
    StubEcho(EchoPrompts),
    StubCapture(CaptureBlocks),
    #[cfg(test)]
    Smoke(SmokeKpopBuilder),
}

impl KpopMultiturnPrompts<'_> {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn kpop_block(&mut self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.kpop_block(),
            Self::StubMt(inner) => inner.kpop_block(),
            Self::StubEcho(inner) => inner.kpop_block(),
            Self::StubCapture(inner) => inner.kpop_block(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("k".to_string()),
        }
    }
}

#[cfg(test)]
impl SmokeKpopBuilder {
    #[allow(dead_code)]
    pub(crate) const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod kpop_multiturn_prompts_tests {
    include!("kpop_multiturn_prompts_tests.inc");
}
