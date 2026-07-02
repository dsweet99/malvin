use std::path::PathBuf;

use super::counters::{hypotheses_emitted, read_exp_log_text};
use crate::kpop_progression::{mpc_plan_declares_done, strip_mpc_plan_done_on_disk};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;

use super::multiturn_types::KpopMultiturnParams;

pub struct KpopMultiturnState<'a> {
    pub(crate) builder: KpopMultiturnPrompts<'a>,
    pub(crate) exp_log_path: PathBuf,
    pub(crate) mpc_plan_path: PathBuf,
    pub max_hypotheses: usize,
    pub(crate) prompt_sent: bool,
    pub(crate) done: bool,
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
        max_hypotheses: usize,
    ) -> Result<Self, String> {
        Self::from_params(KpopMultiturnParams {
            builder,
            exp_log_path,
            mpc_plan_path,
            max_hypotheses,
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
            max_hypotheses: params.max_hypotheses,
            prompt_sent: false,
            done: false,
        })
    }

    /// Returns the next prompt to send, or `None` when the multiturn session should stop.
    ///
    /// # Errors
    ///
    /// Returns `Err` when reading the log or building prompt text fails.
    pub fn next_prompt(&mut self) -> Result<Option<MultiturnPrompt>, String> {
        if self.done {
            return Ok(None);
        }
        let text = read_exp_log_text(&self.exp_log_path)?;
        if mpc_plan_declares_done(&self.mpc_plan_path).unwrap_or(false) {
            self.done = true;
            return Ok(None);
        }
        if hypotheses_emitted(&text) >= self.max_hypotheses {
            self.done = true;
            return Ok(None);
        }
        if self.prompt_sent {
            self.done = true;
            return Ok(None);
        }
        self.prompt_sent = true;
        let remaining_after = self
            .max_hypotheses
            .saturating_sub(hypotheses_emitted(&text));
        self.builder
            .kpop_block(self.max_hypotheses, remaining_after)
            .map(|s| Some(MultiturnPrompt::KpopBlock(s)))
    }

    pub const fn record_kpop_block_prompt_completed(&mut self) {
        // Single-prompt sessions: no catch-up rounds.
    }

    /// Clears the in-flight prompt latch after a failed ACP transport attempt so the outer
    /// retry loop can call [`Self::next_prompt`] again.
    ///
    /// Strips any stale mpc plan `DONE` marker written during the failed attempt so a retry
    /// cannot short-circuit to success without restore, budget checks, or a fresh agent pass.
    pub(crate) fn reset_for_transport_retry(&mut self) {
        self.prompt_sent = false;
        self.done = false;
        strip_mpc_plan_done_on_disk(&self.mpc_plan_path);
    }
}

#[cfg(test)]
#[path = "multiturn_transport_retry_tests.rs"]
mod multiturn_transport_retry_tests;
