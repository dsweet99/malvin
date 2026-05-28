use std::collections::HashMap;

use crate::prompts::{PromptError, PromptStore, render_header};

#[derive(Debug)]
pub struct KpopTurnPrompts<'a> {
    pub store: &'a PromptStore,
    pub base: &'a HashMap<String, String>,
    pub request_text: &'a str,
    pub prepend_rules_once: bool,
}

impl KpopTurnPrompts<'_> {
    fn render_turn_with_body(
        &self,
        body_file: &str,
        ctx: &HashMap<String, String>,
        with_rules: bool,
    ) -> Result<String, String> {
        let common = self
            .store
            .render_prompt_only("kpop_common.md", ctx)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only(body_file, ctx)
            .map_err(|e: PromptError| e.0)?;
        let rules = if with_rules {
            Some(render_header(self.store, ctx).map_err(|e: PromptError| e.0)?)
        } else {
            None
        };
        rules.map_or_else(
            || Ok(format!("{}\n\n{}", common.trim_end(), body.trim_end())),
            |rules| {
                Ok(format!(
                    "{}\n\n{}\n\n{}",
                    rules.trim_end(),
                    common.trim_end(),
                    body.trim_end()
                ))
            },
        )
    }

    /// Gate workflow: `header.md` + `kpop_common.md` + `kpop_block.md` in one prompt (`want` = budget).
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn gate_kpop_single_turn_prompt(&self, max_hypotheses: usize) -> Result<String, String> {
        let mut ctx = self.base.clone();
        ctx.insert("want".to_string(), max_hypotheses.to_string());
        ctx.insert("remaining_hypotheses".to_string(), "0".to_string());
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
        Ok(format!(
            "{}\n\n{}\n\n{}",
            header.trim_end(),
            common.trim_end(),
            body.trim_end()
        ))
    }

    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block(
        &mut self,
        want: usize,
        remaining_after_this_turn: usize,
    ) -> Result<String, String> {
        let mut ctx = self.base.clone();
        ctx.insert("want".to_string(), want.to_string());
        ctx.insert(
            "remaining_hypotheses".to_string(),
            remaining_after_this_turn.to_string(),
        );
        ctx.insert("user_request".to_string(), self.request_text.to_string());
        let with_rules = self.prepend_rules_once;
        let prompt = self.render_turn_with_body("kpop_block.md", &ctx, with_rules)?;
        self.prepend_rules_once = false;
        Ok(prompt)
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
