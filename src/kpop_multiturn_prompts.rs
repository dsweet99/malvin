use crate::kpop_test_stubs::{CaptureWants, EchoPrompts, MtStubPrompts};
use crate::kpop_turn_prompts::KpopTurnPrompts;

#[cfg(test)]
#[derive(Debug)]
pub struct SmokeKpopBuilder;

#[derive(Debug)]
pub enum KpopMultiturnPrompts<'a> {
    Turn(KpopTurnPrompts<'a>),
    StubMt(MtStubPrompts),
    StubEcho(EchoPrompts),
    StubCapture(CaptureWants),
    #[cfg(test)]
    Smoke(SmokeKpopBuilder),
}

impl KpopMultiturnPrompts<'_> {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn kpop_block(
        &mut self,
        want: usize,
        remaining_after_this_turn: usize,
    ) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.kpop_block(want, remaining_after_this_turn),
            Self::StubMt(inner) => inner.kpop_block(want, remaining_after_this_turn),
            Self::StubEcho(inner) => inner.kpop_block(want, remaining_after_this_turn),
            Self::StubCapture(inner) => inner.kpop_block(want, remaining_after_this_turn),
            #[cfg(test)]
            Self::Smoke(_) => Ok("k".to_string()),
        }
    }

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn mbc2_turn(&mut self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.mbc2_turn(),
            Self::StubMt(inner) => inner.mbc2_turn(),
            Self::StubEcho(inner) => inner.mbc2_turn(),
            Self::StubCapture(inner) => inner.mbc2_turn(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("m".to_string()),
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
