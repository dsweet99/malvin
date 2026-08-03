// Embedded default prompt bodies (`default_prompts/`).

#[path = "default_files.rs"]
mod default_files;

pub use default_files::default_file;

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";
pub const ROUTER_A_MD: &str = "router_a.md";
pub const ROUTER_B_MD: &str = "router_b.md";
pub const ROUTER_CODE_EXTRA_MD: &str = "router_code_extra.md";
pub const ROUTER_SUMMARIZE_MD: &str = "router_summarize.md";
pub const EXPLAIN_WRAPPER_MD: &str = "explain_wrapper.md";

pub const REQUIRED_PROMPTS: &[&str] = &[HEADER_MD, "kpop_program.md"];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "kpop_common.md",
    "kpop_block.md",
    "mbc2.md",
    "kpop_program.md",
    "kpop_summarize.md",
    "code_constraints.md",
    "init_constraints.md",
    HEADER_MD,
    DO_HEADER_MD,
    ROUTER_A_MD,
    ROUTER_B_MD,
    ROUTER_CODE_EXTRA_MD,
    ROUTER_SUMMARIZE_MD,
    EXPLAIN_WRAPPER_MD,
];

#[cfg(test)]
mod review_plan_embed_tests {
    use super::DEFAULT_PROMPTS;
    use super::default_file;
    use crate::prompts::malformed_brace_placeholders;

    #[test]
    fn embedded_defaults_exclude_mpc_blocks() {
        for name in DEFAULT_PROMPTS {
            assert!(
                !name.starts_with("mpc_block_"),
                "DEFAULT_PROMPTS must not embed mpc_block files: {name}"
            );
        }
        let count = std::fs::read_dir("default_prompts")
            .expect("default_prompts dir")
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("mpc_block_"))
            })
            .count();
        assert_eq!(count, 0, "default_prompts/ must contain no mpc_block_*.md files");
    }

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

    use crate::config::DEFAULT_CLI_MODEL;
    use crate::artifacts::create_run_artifacts;
    use crate::orchestrator::workflow_context_paths_only;
    use crate::prompts::{PromptStore, render_header};

    #[test]
    fn embedded_header_render_without_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .expect("git init");
        crate::seed_malvin_checks(tmp.path(), "true\n");
        let plan_path = tmp.path().join("plan.md");
        std::fs::write(&plan_path, "plan body\n").expect("write plan");
        let artifacts =
            create_run_artifacts(Path::new(&plan_path), Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let ctx = workflow_context_paths_only(&artifacts, DEFAULT_CLI_MODEL, false);
        let header = render_header(&store, ctx.as_map()).expect("header");
        assert!(!header.contains("{{"), "header must expand all placeholders");
        assert!(
            header.contains(".malvin_home/logs"),
            "header must render logs_dir to home logs bucket"
        );
        let workspace = ctx.get("workspace_dir").expect("workspace_dir");
        assert_eq!(
            Some(workspace.as_str()),
            ctx.get("malvin_output_path").map(String::as_str),
            "workspace_dir should be the per-run log directory"
        );
        assert!(
            header.contains(workspace),
            "header must render workspace_dir to the per-run log directory"
        );
        assert!(
            header.contains("User:"),
            "header must render current_state from workflow context"
        );
    }
}

#[cfg(test)]
mod router_header_embed_tests {
    use std::path::Path;

    use super::{
        default_file, DO_HEADER_MD, HEADER_MD, ROUTER_A_MD, ROUTER_B_MD, ROUTER_SUMMARIZE_MD,
    };
    use crate::config::DEFAULT_CLI_MODEL;
    use crate::artifacts::create_run_artifacts;
    use crate::orchestrator::workflow_context_paths_only;
    use crate::prompts::{PromptStore, render_header};
    use crate::router_flow::router_flow_prompt::{
        build_router_a_prompt, build_router_b_prompt, build_router_header_prompt,
        build_router_kpop_common_prompt, build_router_summarize_prompt, prepare_router_prompt_store,
        RouterAPromptInput, RouterBPromptInput, RouterHeaderPromptInput,
        RouterKpopCommonPromptInput, RouterSummarizePromptInput,
    };

    #[test]
    fn embedded_header_and_router_render_without_unresolved_braces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .status()
            .expect("git init");
        crate::seed_malvin_checks(tmp.path(), "true\n");
        let plan_path = tmp.path().join("plan.md");
        std::fs::write(&plan_path, "plan body\n").expect("write plan");
        let artifacts =
            create_run_artifacts(Path::new(&plan_path), Some(tmp.path())).expect("artifacts");
        let store = PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let ctx = workflow_context_paths_only(&artifacts, DEFAULT_CLI_MODEL, false);
        let header = render_header(&store, ctx.as_map()).expect("header");
        assert!(!header.contains("{{"), "header must expand all placeholders");
        let store = prepare_router_prompt_store().expect("store");
        let header_turn = build_router_header_prompt(RouterHeaderPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
        })
        .expect("header turn");
        assert!(!header_turn.contains("{{"));
        let kpop = build_router_kpop_common_prompt(RouterKpopCommonPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
            max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
            exp_log: &artifacts.gate_exp_log_path(1),
        })
        .expect("kpop common");
        assert!(!kpop.contains("{{"));
        let a = build_router_a_prompt(RouterAPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
            gates: false,
        })
        .expect("router_a");
        assert!(!a.contains("{{"));
        let a_gates = build_router_a_prompt(RouterAPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
            gates: true,
        })
        .expect("router_a gates");
        assert!(!a_gates.contains("{{"));
        let b = build_router_b_prompt(RouterBPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
        })
        .expect("router_b");
        assert!(!b.contains("{{"));
        let summarize = build_router_summarize_prompt(RouterSummarizePromptInput {
            store: &store,
            artifacts: &artifacts,
            model: DEFAULT_CLI_MODEL,
            git: false,
        })
        .expect("summarize");
        assert!(!summarize.contains("{{"));
        assert!(
            summarize.contains("Write a summary of this entire session"),
            "router_summarize.md body must be rendered: {summarize}"
        );
        assert!(default_file(ROUTER_A_MD).is_some());
        assert!(default_file(ROUTER_B_MD).is_some());
        assert!(default_file(ROUTER_SUMMARIZE_MD).is_some());
        assert!(default_file(DO_HEADER_MD).is_some());
        assert!(default_file(HEADER_MD).is_some());
    }
}
