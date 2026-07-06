use std::path::PathBuf;

use super::counters::read_exp_log_text;
use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::multiturn_prompt::MultiturnPrompt;

use super::multiturn_types::KpopMultiturnParams;

pub struct KpopMultiturnState<'a> {
    pub(crate) builder: KpopMultiturnPrompts<'a>,
    pub(crate) exp_log_path: PathBuf,
    sent_prompt: bool,
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
    pub fn new(builder: KpopMultiturnPrompts<'a>, exp_log_path: PathBuf) -> Result<Self, String> {
        Self::from_params(KpopMultiturnParams {
            builder,
            exp_log_path,
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
            sent_prompt: false,
        })
    }

    /// Returns the next prompt to send, or `None` when the session should stop.
    ///
    /// # Errors
    ///
    /// Returns `Err` when reading the log or building prompt text fails.
    pub fn next_prompt(&mut self) -> Result<Option<MultiturnPrompt>, String> {
        if self.sent_prompt {
            return Ok(None);
        }
        let _text = read_exp_log_text(&self.exp_log_path)?;
        let prompt = self.builder.kpop_prompt()?;
        self.sent_prompt = true;
        Ok(Some(MultiturnPrompt::KpopBlock(prompt)))
    }

    pub const fn record_kpop_block_prompt_completed(&mut self) {}

    /// Resets so the outer retry loop can call [`Self::next_prompt`] again.
    pub(crate) const fn reset_for_transport_retry(&mut self) {
        self.sent_prompt = false;
    }
}

#[cfg(test)]
#[path = "multiturn_transport_retry_tests.rs"]
mod multiturn_transport_retry_tests;
