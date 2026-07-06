use std::path::PathBuf;

use super::counters::read_exp_log_text;
use crate::kpop_progression::{mpc_plan_declares_done, strip_mpc_plan_done_on_disk};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;

use super::multiturn_types::KpopMultiturnParams;

fn advance_mpc_phase(
    builder: &mut KpopMultiturnPrompts<'_>,
    phase: MpcPhase,
) -> Result<(MpcPhase, String), String> {
    match phase {
        MpcPhase::A => builder.kpop_block_a().map(|s| (MpcPhase::B, s)),
        MpcPhase::B => builder.kpop_block_b().map(|s| (MpcPhase::C, s)),
        MpcPhase::C => builder.kpop_block_c().map(|s| (MpcPhase::Done, s)),
        MpcPhase::Done => Err("advance_mpc_phase called in Done phase".into()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MpcPhase {
    A,
    B,
    C,
    Done,
}

pub struct KpopMultiturnState<'a> {
    pub(crate) builder: KpopMultiturnPrompts<'a>,
    pub(crate) exp_log_path: PathBuf,
    pub(crate) mpc_plan_path: PathBuf,
    phase: MpcPhase,
}

impl<'a> KpopMultiturnState<'a> {
    pub fn exp_log_path(&self) -> &std::path::Path {
        &self.exp_log_path
    }

    /// Constructs state after reading the experiment log on disk.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the experiment log cannot be read.
    pub fn new(
        builder: KpopMultiturnPrompts<'a>,
        exp_log_path: PathBuf,
        mpc_plan_path: PathBuf,
    ) -> Result<Self, String> {
        Self::from_params(KpopMultiturnParams {
            builder,
            exp_log_path,
            mpc_plan_path,
        })
    }

    /// Same as [`Self::new`] with an explicit parameter bundle.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the experiment log cannot be read.
    pub fn from_params(params: KpopMultiturnParams<'a>) -> Result<Self, String> {
        let _ = read_exp_log_text(&params.exp_log_path)?;
        Ok(Self {
            builder: params.builder,
            exp_log_path: params.exp_log_path,
            mpc_plan_path: params.mpc_plan_path,
            phase: MpcPhase::A,
        })
    }

    /// Returns the next prompt to send, or `None` when the multiturn session should stop.
    ///
    /// Cycles through phases A → B → C, checking for `DONE` before each phase.
    ///
    /// # Errors
    ///
    /// Returns `Err` when reading the log or building prompt text fails.
    pub fn next_prompt(&mut self) -> Result<Option<MultiturnPrompt>, String> {
        if self.phase == MpcPhase::Done {
            return Ok(None);
        }
        let _text = read_exp_log_text(&self.exp_log_path)?;
        if mpc_plan_declares_done(&self.mpc_plan_path).unwrap_or(false) {
            self.phase = MpcPhase::Done;
            return Ok(None);
        }
        let (next_phase, prompt) = advance_mpc_phase(&mut self.builder, self.phase)?;
        self.phase = next_phase;
        Ok(Some(MultiturnPrompt::KpopBlock(prompt)))
    }

    pub const fn record_kpop_block_prompt_completed(&mut self) {}

    /// Resets the phase back to A after a failed ACP transport attempt so the outer
    /// retry loop can call [`Self::next_prompt`] again.
    ///
    /// Strips any stale mpc plan `DONE` marker written during the failed attempt so a retry
    /// cannot short-circuit to success without restore or a fresh agent pass.
    pub(crate) fn reset_for_transport_retry(&mut self) {
        self.phase = MpcPhase::A;
        strip_mpc_plan_done_on_disk(&self.mpc_plan_path);
    }
}

#[cfg(test)]
mod mpc_phase_tests {
    use super::MpcPhase;

    #[test]
    fn mpc_phase_derived_traits() {
        let a = MpcPhase::A;
        let b = MpcPhase::B;
        assert_eq!(a, MpcPhase::A);
        assert_eq!(b, b);
        assert_ne!(a, MpcPhase::Done);
        assert_ne!(a, MpcPhase::C);
        let _ = format!("{a:?}");
        let _ = format!("{b:?}");
        let _ = format!("{:?}", MpcPhase::Done);
    }
}

#[cfg(test)]
#[path = "multiturn_transport_retry_tests.rs"]
mod multiturn_transport_retry_tests;
