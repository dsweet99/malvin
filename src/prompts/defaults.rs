// Embedded default prompt bodies (`default_prompts/`).

#[path = "default_files.rs"]
mod default_files;

pub use default_files::default_file;

pub const HEADER_MD: &str = "header.md";
pub const DO_HEADER_MD: &str = "do_header.md";
pub const ROUTER_A_1_MD: &str = "router_a_1.md";
pub const ROUTER_A_2_MD: &str = "router_a_2.md";
pub const ROUTER_B_SIMPLE_MD: &str = "router_b_simple.md";
pub const ROUTER_B_COMPLEX_MD: &str = "router_b_complex.md";
pub const ROUTER_C_MD: &str = "router_c.md";
pub const ROUTER_CODE_EXTRA_MD: &str = "router_code_extra.md";
pub const ROUTER_D_MD: &str = "router_d.md";

pub const REQUIRED_PROMPTS: &[&str] = &[HEADER_MD, "kpop_program.md"];

pub const DEFAULT_PROMPTS: &[&str] = &[
    "kpop_common.md",
    "kpop_block.md",
    "mbc2.md",
    "kpop_program.md",
    "kpop_program_creative.md",
    "kpop_summarize.md",
    "tidy_constraints.md",
    "code_constraints.md",
    "init_constraints.md",
    "delight_constraints.md",
    "priors_constraints.md",
    "revise_constraints.md",
    "mini_constraints.md",
    HEADER_MD,
    DO_HEADER_MD,
    ROUTER_A_1_MD,
    ROUTER_A_2_MD,
    ROUTER_B_SIMPLE_MD,
    ROUTER_B_COMPLEX_MD,
    ROUTER_C_MD,
    ROUTER_CODE_EXTRA_MD,
    ROUTER_D_MD,
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
        let ctx = workflow_context_paths_only(&artifacts, DEFAULT_CLI_MODEL);
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
mod router_header_embed_tests {
    use std::path::Path;

    use super::{
        default_file, DO_HEADER_MD, HEADER_MD, ROUTER_A_1_MD, ROUTER_A_2_MD, ROUTER_B_COMPLEX_MD,
        ROUTER_B_SIMPLE_MD, ROUTER_C_MD,
    };
    use crate::config::DEFAULT_CLI_MODEL;
    use crate::artifacts::create_run_artifacts;
    use crate::orchestrator::workflow_context_paths_only;
    use crate::prompts::{PromptStore, render_header};
    use crate::router_flow::router_flow_prompt::{
        build_router_a_2_prompt, build_router_b_prompt, build_router_c_prompt,
        build_router_coder_run, prepare_router_prompt_store, RouterBPromptInput,
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
        let ctx = workflow_context_paths_only(&artifacts, DEFAULT_CLI_MODEL);
        let header = render_header(&store, ctx.as_map()).expect("header");
        assert!(!header.contains("{{"), "header must expand all placeholders");
        let paths_ctx = workflow_context_paths_only(&artifacts, DEFAULT_CLI_MODEL);
        let router_a_1 = store
            .render_prompt_only(ROUTER_A_1_MD, paths_ctx.as_map())
            .expect("router_a_1");
        assert!(
            !router_a_1.contains("{{"),
            "router_a_1.md must expand user_request_path"
        );
        let run = build_router_coder_run(&artifacts, "user body", DEFAULT_CLI_MODEL).expect("run");
        assert!(!run.combined.contains("{{"));
        let store = prepare_router_prompt_store().expect("store");
        let router_a_2 = build_router_a_2_prompt(&store, &artifacts, DEFAULT_CLI_MODEL).expect("router_a_2");
        assert!(!router_a_2.contains("{{"));
        assert!(router_a_2.contains("CODING_TASK"));
        let router_b = build_router_b_prompt(RouterBPromptInput {
            store: &store,
            artifacts: &artifacts,
            template: ROUTER_B_SIMPLE_MD,
            coding_task: false,
            model: DEFAULT_CLI_MODEL,
        }).expect("router_b");
        assert!(!router_b.contains("{{"));
        let router_c = build_router_c_prompt(&store, &artifacts, DEFAULT_CLI_MODEL).expect("router_c");
        assert!(!router_c.contains("{{"));
        assert!(
            router_c.contains("still_not_done.md"),
            "router_c must expand still_not_done_path; got:\n{router_c}"
        );
        let router_b_complex = build_router_b_prompt(RouterBPromptInput {
            store: &store,
            artifacts: &artifacts,
            template: ROUTER_B_COMPLEX_MD,
            coding_task: true,
            model: DEFAULT_CLI_MODEL,
        }).expect("router_b_complex");
        assert!(!router_b_complex.contains("{{"));
        assert!(
            router_b_complex.contains("still_not_done.md"),
            "router_b_complex must expand still_not_done_path; got:\n{router_b_complex}"
        );
        assert!(default_file(ROUTER_A_1_MD).is_some());
        assert!(default_file(ROUTER_A_2_MD).is_some());
        assert!(default_file(ROUTER_B_SIMPLE_MD).is_some());
        assert!(default_file(ROUTER_B_COMPLEX_MD).is_some());
        assert!(default_file(ROUTER_C_MD).is_some());
        assert!(default_file(DO_HEADER_MD).is_some());
        assert!(default_file(HEADER_MD).is_some());
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

