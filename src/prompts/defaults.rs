// Embedded default prompt bodies (`default_prompts/`).

#[path = "default_files.rs"]
mod default_files;

pub use default_files::default_file;

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";

pub const REQUIRED_PROMPTS: &[&str] = &[HEADER_MD, "kpop_program.md"];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "kpop_common.md",
    "kpop_block.md",
    "mbc2.md",
    "kpop_program.md",
    "kpop_program_creative.md",
    "kpop_summarize.md",
    "mpc_planner.md",
    "tidy_constraints.md",
    "code_constraints.md",
    "init_constraints.md",
    "delight_constraints.md",
    "revise_constraints.md",
    "mini_constraints.md",
    HEADER_MD,
    DO_HEADER_MD,
];

#[cfg(test)]
mod review_plan_embed_tests {
    use super::DEFAULT_PROMPTS;
    use super::default_file;
    use crate::prompts::malformed_brace_placeholders;

    #[test]
    fn embedded_default_prompts_use_spaced_brace_placeholders() {
        for name in DEFAULT_PROMPTS {
            let text = default_file(name).unwrap_or_else(|| panic!("{name} must be embedded"));
            let bad = malformed_brace_placeholders(text);
            assert!(bad.is_empty(), "{name}: {bad:?}");
        }
    }
}

#[cfg(test)]
mod advice_path_embed_tests {
    use std::path::Path;

    use crate::artifacts::create_run_artifacts;
    use crate::orchestrator::workflow_context;
    use crate::prompts::{PromptStore, render_header};

    #[test]
    fn embedded_header_render_without_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan_path = tmp.path().join("plan.md");
        std::fs::write(&plan_path, "plan body\n").expect("write plan");
        let artifacts =
            create_run_artifacts(Path::new(&plan_path), Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
        let header = render_header(&store, ctx.as_map()).expect("header");
        assert!(!header.contains("{{"), "header must expand all placeholders");
        assert!(
            header.contains(".malvin_home/logs"),
            "header must render logs_dir to home logs bucket"
        );
        assert!(
            header.contains("User:"),
            "header must render current_state from workflow context"
        );
    }
}

#[cfg(test)]
mod do_header_tests {
    use super::DO_HEADER_MD;
    use super::default_file;

    #[test]
    fn embedded_do_header_is_a_single_text_block_with_closing_newline() {
        let s = default_file(DO_HEADER_MD).expect("do header must be embedded");
        let lower = s.to_ascii_lowercase();
        assert!(s.ends_with('\n'));
        assert!(lower.contains("no stream of consciousness"));
        assert!(lower.contains("do not restate"));
        assert!(!lower.contains("user request is:"));
        assert!(!s.contains("You'll\n find"));
    }
}

