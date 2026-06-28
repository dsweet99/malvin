use std::collections::HashMap;

use crate::prompt_stratification::join_strata;
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
            || Ok(join_strata([&common, &body])),
            |rules| Ok(join_strata([&rules, &common, &body])),
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
        Ok(join_strata([&header, &common, &body]))
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
mod inline_render_turn_with_body {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn render_turn_with_body_renders_common_and_block() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("prompts");
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(root.join("kpop_common.md"), "common {{ want }}\n").expect("write");
        std::fs::write(root.join("kpop_block.md"), "block {{ user_request }}\n").expect("write");
        let store = crate::prompts::PromptStore::with_root(root);
        store.ensure_defaults().expect("defaults");
        let base = HashMap::from([("plan_path".to_string(), "p".to_string())]);
        let ctx = HashMap::from([
            ("want".to_string(), "1".to_string()),
            ("remaining_hypotheses".to_string(), "0".to_string()),
            ("user_request".to_string(), "inline".to_string()),
        ]);
        let prompts = KpopTurnPrompts {
            store: &store,
            base: &base,
            request_text: "inline",
            prepend_rules_once: false,
        };
        let out = prompts
            .render_turn_with_body("kpop_block.md", &ctx, false)
            .expect("render");
        assert!(out.contains("inline"));
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
