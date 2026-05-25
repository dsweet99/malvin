use std::path::PathBuf;

use rand::SeedableRng;
use rand::rngs::StdRng;

use super::counters::{
    agent_declared_success, block_mean_from_p_creative, hypotheses_emitted, poisson_block_size,
    read_exp_log_text,
};
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;

use super::multiturn_types::{KpopMultiturnParams, NextStep, Phase};

pub struct KpopMultiturnState<'a> {
    pub(crate) builder: KpopMultiturnPrompts<'a>,
    pub(crate) exp_log_path: PathBuf,
    pub max_hypotheses: usize,
    pub p_creative: f64,
    pub(crate) rng: StdRng,
    pub(crate) credit: usize,
    pub(crate) phase: Phase,
    pub(crate) done: bool,
    pub(crate) last_block_miss: Option<super::block_report::KpopBlockMissSnapshot>,
}

use super::multiturn_phases::{run_kpop_phase, run_mbc2_phase};

impl<'a> KpopMultiturnState<'a> {
    pub fn exp_log_path(&self) -> &std::path::Path {
        &self.exp_log_path
    }

    /// Constructs state after reading the experiment log on disk.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the experiment log cannot be read or parsed for the initial phase.
    pub fn new(
        builder: KpopMultiturnPrompts<'a>,
        exp_log_path: PathBuf,
        max_hypotheses: usize,
        p_creative: f64,
    ) -> Result<Self, String> {
        Self::from_params(KpopMultiturnParams {
            builder,
            exp_log_path,
            max_hypotheses,
            p_creative,
            rng: StdRng::from_entropy(),
        })
    }

    /// Same as [`Self::new`] but accepts an explicit RNG and builder bundle.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the experiment log cannot be read or parsed for the initial phase.
    pub fn from_params(mut params: KpopMultiturnParams<'a>) -> Result<Self, String> {
        let text = read_exp_log_text(&params.exp_log_path)?;
        let hypotheses_before = hypotheses_emitted(&text);
        let mean = block_mean_from_p_creative(params.p_creative);
        let n = poisson_block_size(&mut params.rng, mean).max(1);
        let phase = Phase::KpopBlock {
            target_n: n,
            hypotheses_before,
            attempts: 0,
        };
        Ok(Self {
            builder: params.builder,
            exp_log_path: params.exp_log_path,
            max_hypotheses: params.max_hypotheses,
            p_creative: params.p_creative,
            rng: params.rng,
            credit: 0,
            phase,
            done: false,
            last_block_miss: None,
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
        if agent_declared_success(&text) {
            self.done = true;
            return Ok(None);
        }
        if hypotheses_emitted(&text) >= self.max_hypotheses {
            self.done = true;
            return Ok(None);
        }

        let step = match &mut self.phase {
            Phase::KpopBlock { .. } => run_kpop_phase(self, &text)?,
            Phase::Mbc2 { .. } => run_mbc2_phase(self, &text)?,
        };
        match step {
            NextStep::Stop => Ok(None),
            NextStep::Again => self.next_prompt(),
            NextStep::Emit(s) => Ok(Some(s)),
        }
    }

    pub const fn record_kpop_block_prompt_completed(&mut self) {
        if let Phase::KpopBlock { attempts, .. } = &mut self.phase {
            *attempts += 1;
        }
    }

    pub const fn record_mbc2_prompt_completed(&mut self) {
        if let Phase::Mbc2 { sent, .. } = &mut self.phase {
            *sent = if *sent == 0 { 1 } else { 2 };
        }
    }
}
