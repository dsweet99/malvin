//! Per-turn prompt assembly for `KPop` investigation and gate-engine sessions.

use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore, render_header};

#[derive(Debug)]
pub struct KpopTurnPrompts<'a> {
    pub store: &'a PromptStore,
    pub base: &'a WorkflowRenderContext,
    pub request_text: &'a str,
    pub prepend_rules_once: bool,
}

impl KpopTurnPrompts<'_> {
    /// Gate workflow: `header.md` + `kpop_common.md` + `kpop_block.md` in one prompt.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_engine_single_turn_prompt(&self, max_hypotheses: usize) -> Result<String, String> {
        self.gate_kpop_single_turn_prompt(max_hypotheses)
    }

    /// Gate workflow: `header.md` + `kpop_common.md` + `kpop_block.md` in one prompt.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn gate_kpop_single_turn_prompt(&self, max_hypotheses: usize) -> Result<String, String> {
        let mut ctx = self.base.as_map().clone();
        ctx.insert("max_hypotheses".to_string(), max_hypotheses.to_string());
        ctx.insert("user_request".to_string(), self.request_text.to_string());
        let header = self
            .store
            .render_prompt_only("header.md", &ctx)
            .map_err(|e: PromptError| e.0)?;
        let common = self
            .store
            .render_prompt_only("kpop_common.md", &ctx)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only("kpop_block.md", &ctx)
            .map_err(|e: PromptError| e.0)?;
        Ok(join_labeled_strata([
            (PromptStratum::WorkflowHeader, header),
            (PromptStratum::EmbeddedTemplate, common),
            (PromptStratum::GateLoopBlock, body),
        ]))
    }

    /// Investigation turn: optional `header.md` (once) + `kpop_common.md` + `kpop_block.md`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block(&mut self, max_hypotheses: usize) -> Result<String, String> {
        let mut ctx = self.base.as_map().clone();
        ctx.insert("max_hypotheses".to_string(), max_hypotheses.to_string());
        ctx.insert("user_request".to_string(), self.request_text.to_string());
        let with_rules = self.prepend_rules_once;
        let common = self
            .store
            .render_prompt_only("kpop_common.md", &ctx)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only("kpop_block.md", &ctx)
            .map_err(|e: PromptError| e.0)?;
        let rules = if with_rules {
            Some(render_header(self.store, &ctx).map_err(|e: PromptError| e.0)?)
        } else {
            None
        };
        let prompt = rules.map_or_else(
            || format!("{}\n\n{}", common.trim_end(), body.trim_end()),
            |rules| {
                format!(
                    "{}\n\n{}\n\n{}",
                    rules.trim_end(),
                    common.trim_end(),
                    body.trim_end()
                )
            },
        );
        self.prepend_rules_once = false;
        Ok(prompt)
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
