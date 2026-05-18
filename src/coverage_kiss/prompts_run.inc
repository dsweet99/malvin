// Prompt store, local file helper, and inactive memory containment.

use crate::prompts::{KpopPromptValidation, PromptStore};

fn exercise_validations(store: &PromptStore) {
    store.validate_exists("implement.md").expect("exists");
    store
        .validate_kpop_prompts(KpopPromptValidation {
            run_learn: true,
            require_mbc2: true,
        })
        .expect("kpop prompts");
}

fn exercise_renders(store: &PromptStore, ctx: &std::collections::HashMap<String, String>) {
    let _ = store.render("implement.md", ctx).expect("render");
    let _ = store
        .render_prompt_only("kpop_common.md", ctx)
        .expect("render prompt only");
}

#[test]
fn smoke_kiss_write_same_body_files() {
    let tmp = kiss_test_tmp();
    let d = tmp.path().join("prompts");
    kiss_write_same_body_files(&d, &["a.md", "b.md"], b"same");
    assert_eq!(std::fs::read_to_string(d.join("a.md")).unwrap(), "same");
    assert_eq!(std::fs::read_to_string(d.join("b.md")).unwrap(), "same");
}

#[test]
fn smoke_prompt_store_with_root() {
    let tmp = kiss_test_tmp();
    let prompts = tmp.path().join("prompts");
    let store = kiss_smoke_prompt_store(&prompts);
    exercise_validations(&store);
    let store_default = crate::prompts::PromptStore::default_store();
    let _ = crate::prompts::default_file("implement.md").expect("embedded");
    let _ = crate::prompts::user_home_dir();
    let ctx = kiss_smoke_render_context();
    exercise_renders(&store, &ctx);
    let wf_tmp = tempfile::tempdir().expect("tempdir");
    let art = crate::artifacts::create_run_artifacts_from_text("plan", Some(wf_tmp.path()))
        .expect("wf art");
    crate::orchestrator::workflow_context(&art, &store_default, "code").expect("workflow ctx");
}

#[test]
fn smoke_memory_containment_inactive() {
    let inactive = crate::acp_memory_containment::AcpMemoryContainment::inactive();
    let msg = crate::acp_memory_containment::map_acp_child_exit_message(&inactive, "fallback");
    assert_eq!(msg, "fallback");
    crate::acp_memory_containment::finalize_containment_cgroup(&inactive);
}
