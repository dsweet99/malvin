use std::collections::HashMap;

use crate::kpop_turn_prompts::KpopTurnPrompts;
use crate::prompt_stratification::WorkflowRenderContext;
use crate::prompts::{PromptStore, render_header, render_priors_mbc2_prompt};

fn kpop_turn_test_context() -> WorkflowRenderContext {
    WorkflowRenderContext::from(HashMap::from([
        (
            "plan_path".to_string(),
            "plan/with $ and\nmulti-line path".to_string(),
        ),
        ("advice_path".to_string(), "./.malvin/advice.md".to_string()),
        ("logs_dir".to_string(), "./.malvin/logs/run42".to_string()),
        ("exp_log".to_string(), "./.malvin/logs/run/_kpop/exp_log.md".to_string()),
        (
            "user_request_path".to_string(),
            "./.malvin/logs/run/request.md".to_string(),
        ),
        (
            "current_state".to_string(),
            "User: test\nRetry: not a retry".to_string(),
        ),
        (
            "priors_path".to_string(),
            "./.malvin/logs/run/_kpop/priors.md".to_string(),
        ),
    ]))
}

fn kpop_turn_test_store() -> (tempfile::TempDir, PromptStore) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path().join("prompts");
    std::fs::create_dir_all(&root).expect("mkdir");
    for (name, body) in [
        ("header.md", "<<hdr plan={{ plan_path }}>>\n"),
        ("kpop_common.md", "<<common>>\n"),
        ("mpc_block_a.md", "<<block_a req={{ user_request_path }}>>\n"),
        ("mpc_block_b.md", "<<block_b>>\n"),
        ("mpc_block_c.md", "<<block_c>>\n"),
        ("mbc2.md", "MBC2 {{ user_prompt }}\n"),
        (
            "priors.md",
            "Read {{ user_request_path }}. Write {{ priors_path }}.\n",
        ),
    ] {
        std::fs::write(root.join(name), body).expect("write");
    }
    let store = PromptStore::with_root(root);
    store.ensure_defaults().expect("defaults");
    (tmp, store)
}

fn expected_kpop_block_output(
    store: &PromptStore,
    ctx: &WorkflowRenderContext,
    with_rules: bool,
) -> String {
    let map = ctx.as_map();
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .expect("common");
    let body_a = store
        .render_prompt_only("mpc_block_a.md", map)
        .expect("block_a");
    let body_b = store
        .render_prompt_only("mpc_block_b.md", map)
        .expect("block_b");
    let body_c = store
        .render_prompt_only("mpc_block_c.md", map)
        .expect("block_c");
    let bodies = format!(
        "{}\n\n{}\n\n{}",
        body_a.trim_end(),
        body_b.trim_end(),
        body_c.trim_end()
    );
    if with_rules {
        let header = render_header(store, map).expect("header");
        format!(
            "{}\n\n{}\n\n{}",
            header.trim_end(),
            common.trim_end(),
            bodies
        )
    } else {
        format!("{}\n\n{}", common.trim_end(), bodies)
    }
}

#[test]
fn render_turn_with_body_matches_kpop_engine_single_turn_without_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let request_path = "./.malvin/logs/run/request.md";
    let prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: false,
    };
    let gate = prompts.kpop_engine_single_turn_prompt().expect("gate prompt");
    let map = base.as_map();
    let header = store
        .render_prompt_only("header.md", map)
        .expect("header");
    let common = store
        .render_prompt_only("kpop_common.md", map)
        .expect("common");
    let body_a = store
        .render_prompt_only("mpc_block_a.md", map)
        .expect("block_a");
    let body_b = store
        .render_prompt_only("mpc_block_b.md", map)
        .expect("block_b");
    let body_c = store
        .render_prompt_only("mpc_block_c.md", map)
        .expect("block_c");
    let expected = format!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}",
        header.trim_end(),
        common.trim_end(),
        body_a.trim_end(),
        body_b.trim_end(),
        body_c.trim_end()
    );
    assert_eq!(gate, expected);
    assert!(gate.contains(request_path));
    assert!(
        !gate.contains("budget for any KPOPs"),
        "gate prompt must not include hypothesis budget wording"
    );
}

#[test]
fn kpop_block_matches_independently_rendered_sections() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: true,
    };

    let first = prompts.kpop_block().expect("first kpop turn");
    assert_eq!(
        first,
        expected_kpop_block_output(&store, &base, true),
        "first turn should equal header + common + block with exact composition"
    );

    let second = prompts.kpop_block().expect("second kpop turn");
    assert_eq!(
        second,
        expected_kpop_block_output(&store, &base, false),
        "after prepend_rules_once is consumed, output should omit header"
    );
}

#[test]
fn kpop_block_without_prepend_rules_never_includes_header() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let mut prompts = KpopTurnPrompts {
        store: &store,
        base: &base,
        prepend_rules_once: false,
    };

    let out = prompts.kpop_block().expect("kpop turn");
    assert_eq!(
        out,
        expected_kpop_block_output(&store, &base, false),
        "prepend_rules_once=false should never prepend header"
    );
    let header = render_header(&store, base.as_map()).expect("header");
    assert!(
        !out.contains(header.trim()),
        "output must not contain rendered header fragment:\nheader={header:?}\nout={out:?}"
    );
}

#[test]
fn kpop_priors_phase_expands_paths_and_wraps_mbc2() {
    let (_tmp, store) = kpop_turn_test_store();
    let base = kpop_turn_test_context();
    let out = render_priors_mbc2_prompt(&store, base.as_map()).expect("priors phase");
    assert!(out.contains("./.malvin/logs/run/request.md"));
    assert!(out.contains("./.malvin/logs/run/_kpop/priors.md"));
    assert!(out.contains("MBC2"));
    assert!(
        !out.contains("{{"),
        "priors phase must not leave unresolved placeholders: {out}"
    );
}
