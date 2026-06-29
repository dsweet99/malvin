//! Per-turn prompt assembly for **`KPopEngine`** sessions.
//!
//! - **`KPop`** (`kpop_common.md`): agent-side Popper method (Hypothesize → Predict → Falsify).
//! - **`KPopEngine` turn** (`kpop_block.md`): per-iteration budget and `{{ user_request_path }}` (from `base` context).

use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore, render_header};

#[derive(Debug)]
pub struct KpopTurnPrompts<'a> {
    pub store: &'a PromptStore,
    pub base: &'a WorkflowRenderContext,
    pub prepend_rules_once: bool,
}

impl KpopTurnPrompts<'_> {
    fn render_turn_with_body(
        &self,
        body_file: &str,
        ctx: &WorkflowRenderContext,
        with_rules: bool,
    ) -> Result<String, String> {
        let map = ctx.as_map();
        let common = self
            .store
            .render_prompt_only("kpop_common.md", map)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only(body_file, map)
            .map_err(|e: PromptError| e.0)?;
        let rules = if with_rules {
            Some(render_header(self.store, map).map_err(|e: PromptError| e.0)?)
        } else {
            None
        };
        rules.map_or_else(
            || {
                Ok(join_labeled_strata([
                    (PromptStratum::EmbeddedTemplate, &common),
                    (PromptStratum::GateLoopBlock, &body),
                ]))
            },
            |rules| {
                Ok(join_labeled_strata([
                    (PromptStratum::WorkflowHeader, &rules),
                    (PromptStratum::EmbeddedTemplate, &common),
                    (PromptStratum::GateLoopBlock, &body),
                ]))
            },
        )
    }

    /// Gate workflow: `header.md` + `kpop_common.md` + `kpop_block.md` in one prompt (`want` = budget).
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_engine_single_turn_prompt(&self, max_hypotheses: usize) -> Result<String, String> {
        let mut ctx = self.base.clone();
        ctx.insert("want".to_string(), max_hypotheses.to_string());
        ctx.insert("remaining_hypotheses".to_string(), "0".to_string());
        let map = ctx.as_map();
        let header = self
            .store
            .render_prompt_only("header.md", map)
            .map_err(|e: PromptError| e.0)?;
        let common = self
            .store
            .render_prompt_only("kpop_common.md", map)
            .map_err(|e: PromptError| e.0)?;
        let body = self
            .store
            .render_prompt_only("kpop_block.md", map)
            .map_err(|e: PromptError| e.0)?;
        Ok(join_labeled_strata([
            (PromptStratum::WorkflowHeader, header),
            (PromptStratum::EmbeddedTemplate, common),
            (PromptStratum::GateLoopBlock, body),
        ]))
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
        let with_rules = self.prepend_rules_once;
        let prompt = self.render_turn_with_body("kpop_block.md", &ctx, with_rules)?;
        self.prepend_rules_once = false;
        Ok(prompt)
    }
}

#[cfg(test)]
mod inline_render_turn_with_body {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn render_turn_with_body_renders_common_and_block() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("prompts");
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(root.join("kpop_common.md"), "common {{ want }}\n").expect("write");
        std::fs::write(root.join("kpop_block.md"), "block {{ user_request_path }}\n").expect("write");
        let store = crate::prompts::PromptStore::with_root(root);
        store.ensure_defaults().expect("defaults");
        let base = WorkflowRenderContext::from(HashMap::from([
            ("plan_path".to_string(), "p".to_string()),
            ("user_request_path".to_string(), "./req.md".to_string()),
        ]));
        let ctx = WorkflowRenderContext::from(HashMap::from([
            ("want".to_string(), "1".to_string()),
            ("remaining_hypotheses".to_string(), "0".to_string()),
            ("user_request_path".to_string(), "./req.md".to_string()),
        ]));
        let prompts = KpopTurnPrompts {
            store: &store,
            base: &base,
            prepend_rules_once: false,
        };
        let out = prompts
            .render_turn_with_body("kpop_block.md", &ctx, false)
            .expect("render");
        assert!(out.contains("./req.md"));
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
