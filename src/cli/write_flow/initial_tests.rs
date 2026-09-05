use super::{WriteInitialPromptInput, build_write_workflow_initial_prompt, compose_write_b_prompt};
use crate::prompts::{HEADER_MD, PromptStore, WRITE_A_MD};
use crate::test_utils::with_isolated_home;

fn write_minimal_write_prompts(prompt_root: &std::path::Path) {
    std::fs::create_dir_all(prompt_root).expect("mkdir");
    std::fs::write(
        prompt_root.join(HEADER_MD),
        "HEADER git={{ git_extra }} cmd={{ malvin_command }}\n",
    )
    .expect("header");
    std::fs::write(
        prompt_root.join(WRITE_A_MD),
        "WRITE_A {{ request_text }} in {{ workspace_dir }}\n",
    )
    .expect("write_a");
}

#[test]
fn write_initial_joins_header_and_write_a_in_order() {
    with_isolated_home(|work| {
        let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
            "topic",
            Some(work),
            crate::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_write_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let out = build_write_workflow_initial_prompt(WriteInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            git: false,
            request_text: "how gates exit",
            workspace_dir: "./ws",
        })
        .expect("initial");
        assert!(out.body.contains("HEADER"));
        assert!(out.body.contains("WRITE_A"));
        assert!(out.body.contains("how gates exit"));
        assert!(out.body.contains("./ws"));
        let header_at = out.body.find("HEADER").expect("header");
        let a_at = out.body.find("WRITE_A").expect("a");
        assert!(header_at < a_at);
        assert_eq!(out.stdout_label, format!("{HEADER_MD}+{WRITE_A_MD}"));
        assert_eq!(out.log_who, "write_initial");
    });
}

#[test]
fn write_initial_respects_git_in_header_not_out_path() {
    with_isolated_home(|work| {
        let artifacts = crate::artifacts::create_run_artifacts_from_text_opts(
            "topic",
            Some(work),
            crate::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_write_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let with_git = build_write_workflow_initial_prompt(WriteInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:sonnet-4",
            git: true,
            request_text: "topic",
            workspace_dir: "./ws",
        })
        .expect("git");
        let no_git = build_write_workflow_initial_prompt(WriteInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:sonnet-4",
            git: false,
            request_text: "topic",
            workspace_dir: "./ws",
        })
        .expect("no_git");
        assert_ne!(with_git.body, no_git.body);
        assert!(with_git.body.contains("cursor:sonnet-4"));
        assert!(with_git.body.contains(crate::workflow_context::GIT_EXTRA_ENABLED));
        assert!(!no_git.body.contains(crate::workflow_context::GIT_EXTRA_ENABLED));
        assert!(!with_git.body.contains("write.tex"));
        assert!(!with_git.body.contains("write.pdf"));
        assert!(!with_git.body.contains("mbc2"));
        assert!(!with_git.body.contains("router_a"));
        assert!(!with_git.body.contains("kpop"));
    });
}

#[test]
fn write_b_carries_out_path_not_initial() {
    let b = compose_write_b_prompt(
        &crate::prompts::PromptStore::default_store(),
        "docs/paper.tex",
        "docs/paper.pdf",
        "./.malvin_home/logs/run",
    )
    .expect("write_b");
    assert!(b.contains("`docs/paper.tex`"));
    assert!(b.contains("`docs/paper.pdf`"));
}
