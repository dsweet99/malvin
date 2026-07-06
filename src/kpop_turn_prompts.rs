//! Per-turn prompt assembly for **`KPopEngine`** sessions.
//!
//! - **`KPop`** (`kpop_common.md`): agent-side Popper method (Hypothesize → Predict → Falsify).
//! - **`KPopEngine` turn** (`mpc_block_a/b/c.md`): per-iteration MPC plan workflow split into
//!   three sequential sub-prompts within one conversation.

use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore, render_header};

const MPC_BLOCK_FILES: [&str; 3] = ["mpc_block_a.md", "mpc_block_b.md", "mpc_block_c.md"];

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

    /// Gate workflow: `header.md` + `kpop_common.md` + all three `mpc_block` files in one prompt.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_engine_single_turn_prompt(&self) -> Result<String, String> {
        let map = self.base.as_map();
        let header = self
            .store
            .render_prompt_only("header.md", map)
            .map_err(|e: PromptError| e.0)?;
        let common = self
            .store
            .render_prompt_only("kpop_common.md", map)
            .map_err(|e: PromptError| e.0)?;
        let mut parts = vec![
            (PromptStratum::WorkflowHeader, header),
            (PromptStratum::EmbeddedTemplate, common),
        ];
        for file in MPC_BLOCK_FILES {
            let body = self
                .store
                .render_prompt_only(file, map)
                .map_err(|e: PromptError| e.0)?;
            parts.push((PromptStratum::GateLoopBlock, body));
        }
        Ok(join_labeled_strata(parts))
    }

    /// Multi-turn phase A: `kpop_common.md` + `mpc_block_a.md` (with optional header).
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block_a(&mut self) -> Result<String, String> {
        let with_rules = self.prepend_rules_once;
        let prompt = self.render_turn_with_body("mpc_block_a.md", self.base, with_rules)?;
        self.prepend_rules_once = false;
        Ok(prompt)
    }

    /// Multi-turn phase B: `mpc_block_b.md` only.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block_b(&self) -> Result<String, String> {
        let map = self.base.as_map();
        self.store
            .render_prompt_only("mpc_block_b.md", map)
            .map_err(|e: PromptError| e.0)
    }

    /// Multi-turn phase C: `mpc_block_c.md` only.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block_c(&self) -> Result<String, String> {
        let map = self.base.as_map();
        self.store
            .render_prompt_only("mpc_block_c.md", map)
            .map_err(|e: PromptError| e.0)
    }

    /// Concatenated single-turn prompt: `kpop_common.md` + all three `mpc_block` files.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_block(&mut self) -> Result<String, String> {
        let with_rules = self.prepend_rules_once;
        let map = self.base.as_map();
        let common = self
            .store
            .render_prompt_only("kpop_common.md", map)
            .map_err(|e: PromptError| e.0)?;
        self.prepend_rules_once = false;
        let mut parts: Vec<(PromptStratum, String)> = Vec::new();
        if with_rules {
            let rules = render_header(self.store, map).map_err(|e: PromptError| e.0)?;
            parts.push((PromptStratum::WorkflowHeader, rules));
        }
        parts.push((PromptStratum::EmbeddedTemplate, common));
        for file in MPC_BLOCK_FILES {
            let body = self
                .store
                .render_prompt_only(file, map)
                .map_err(|e: PromptError| e.0)?;
            parts.push((PromptStratum::GateLoopBlock, body));
        }
        Ok(join_labeled_strata(parts))
    }

    /// Multi-turn engine prompt for phase A: `header.md` + `kpop_common.md` + `mpc_block_a.md`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_engine_prompt_a(&self) -> Result<String, String> {
        let map = self.base.as_map();
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
            .render_prompt_only("mpc_block_a.md", map)
            .map_err(|e: PromptError| e.0)?;
        Ok(join_labeled_strata([
            (PromptStratum::WorkflowHeader, header),
            (PromptStratum::EmbeddedTemplate, common),
            (PromptStratum::GateLoopBlock, body),
        ]))
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
        std::fs::write(root.join("kpop_common.md"), "common\n").expect("write");
        std::fs::write(root.join("mpc_block_a.md"), "block {{ user_request_path }}\n")
            .expect("write");
        let store = crate::prompts::PromptStore::with_root(root);
        store.ensure_defaults().expect("defaults");
        let base = WorkflowRenderContext::from(HashMap::from([
            ("plan_path".to_string(), "p".to_string()),
            ("user_request_path".to_string(), "./req.md".to_string()),
        ]));
        let ctx = WorkflowRenderContext::from(HashMap::from([(
            "user_request_path".to_string(),
            "./req.md".to_string(),
        )]));
        let prompts = KpopTurnPrompts {
            store: &store,
            base: &base,
            prepend_rules_once: false,
        };
        let out = prompts
            .render_turn_with_body("mpc_block_a.md", &ctx, false)
            .expect("render");
        assert!(out.contains("./req.md"));
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
