//! Per-turn prompt assembly for **`KPopEngine`** sessions.
//!
//! - **`KPop`** (`kpop_common.md`): agent-side Popper method (Hypothesize → Predict → Falsify).

use crate::prompt_stratification::{join_labeled_strata, PromptStratum, WorkflowRenderContext};
use crate::prompts::{PromptError, PromptStore, render_header};

#[derive(Debug)]
pub struct KpopTurnPrompts<'a> {
    pub store: &'a PromptStore,
    pub base: &'a WorkflowRenderContext,
    pub prepend_rules_once: bool,
}

impl KpopTurnPrompts<'_> {
    /// Gate workflow: `header.md` + `kpop_common.md`.
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
        Ok(join_labeled_strata([
            (PromptStratum::WorkflowHeader, header),
            (PromptStratum::EmbeddedTemplate, common),
        ]))
    }

    /// Bare `malvin kpop`: optional `header.md` (once) + `kpop_common.md`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when a prompt template cannot be rendered.
    pub fn kpop_prompt(&mut self) -> Result<String, String> {
        let with_rules = self.prepend_rules_once;
        let map = self.base.as_map();
        let common = self
            .store
            .render_prompt_only("kpop_common.md", map)
            .map_err(|e: PromptError| e.0)?;
        self.prepend_rules_once = false;
        if with_rules {
            let rules = render_header(self.store, map).map_err(|e: PromptError| e.0)?;
            Ok(join_labeled_strata([
                (PromptStratum::WorkflowHeader, &rules),
                (PromptStratum::EmbeddedTemplate, &common),
            ]))
        } else {
            Ok(join_labeled_strata([(PromptStratum::EmbeddedTemplate, &common)]))
        }
    }
}

#[cfg(test)]
mod inline_render_turn_with_body {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn kpop_prompt_renders_common_and_exp_log_placeholders() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path().join("prompts");
        std::fs::create_dir_all(&root).expect("mkdir");
        std::fs::write(root.join("header.md"), "hdr {{ user_request_path }}\n").expect("write");
        std::fs::write(root.join("kpop_common.md"), "common {{ exp_log }}\n").expect("write");
        let store = crate::prompts::PromptStore::with_root(root);
        store.ensure_defaults().expect("defaults");
        let base = WorkflowRenderContext::from(HashMap::from([
            ("plan_path".to_string(), "p".to_string()),
            ("user_request_path".to_string(), "./req.md".to_string()),
            ("exp_log".to_string(), "./_kpop/exp.md".to_string()),
        ]));
        let mut prompts = KpopTurnPrompts {
            store: &store,
            base: &base,
            prepend_rules_once: true,
        };
        let out = prompts.kpop_prompt().expect("render");
        assert!(out.contains("./req.md"));
        assert!(out.contains("./_kpop/exp.md"));
        assert!(out.contains("Hypothesize") || out.contains("common"));
    }
}

#[cfg(test)]
#[path = "kpop_turn_prompts_tests.rs"]
mod kpop_turn_prompts_tests;
