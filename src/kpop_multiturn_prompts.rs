use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};
use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::prompts::render_priors_mbc2_prompt;

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
    pub fn kpop_priors(&self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => render_priors_mbc2_prompt(inner.store, inner.base.as_map())
                .map_err(|e| e.0),
            Self::StubMt(inner) => inner.kpop_priors(),
            Self::StubEcho(inner) => inner.kpop_priors(),
            Self::StubCapture(inner) => inner.kpop_priors(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("priors".to_string()),
        }
    }

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

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn kpop_block_a(&mut self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.kpop_block_a(),
            Self::StubMt(inner) => inner.kpop_block_a(),
            Self::StubEcho(inner) => inner.kpop_block_a(),
            Self::StubCapture(inner) => inner.kpop_block_a(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("ka".to_string()),
        }
    }

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn kpop_block_b(&self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.kpop_block_b(),
            Self::StubMt(inner) => inner.kpop_block_b(),
            Self::StubEcho(inner) => inner.kpop_block_b(),
            Self::StubCapture(inner) => inner.kpop_block_b(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("kb".to_string()),
        }
    }

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails.
    pub fn kpop_block_c(&self) -> Result<String, String> {
        match self {
            Self::Turn(inner) => inner.kpop_block_c(),
            Self::StubMt(inner) => inner.kpop_block_c(),
            Self::StubEcho(inner) => inner.kpop_block_c(),
            Self::StubCapture(inner) => inner.kpop_block_c(),
            #[cfg(test)]
            Self::Smoke(_) => Ok("kc".to_string()),
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
