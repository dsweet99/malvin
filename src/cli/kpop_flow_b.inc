
pub fn kpop_emit_startup(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    super::run_emit::emit_run_startup_sequence(artifacts, shared.tee_startup_stdout(), &kpop.request)
}

pub fn kpop_learn_bundle(
    store: &PromptStore,
    context: &HashMap<String, String>,
    run_learn: bool,
    artifacts: &RunArtifacts,
) -> Result<Option<(String, PathBuf)>, String> {
    if !run_learn {
        return Ok(None);
    }
    let learn_prompt = store
        .render("learn.md", context)
        .map_err(|e: PromptError| e.0)?;
    let learn_log = artifacts.log_path("learn_kpop");
    Ok(Some((learn_prompt, learn_log)))
}

#[test]
fn stringify_kpop_flow_helpers() {
    let _ = crate::acp_post_run::merge_acp_with_workspace_session_restore;
    let _ = crate::cli::kpop_flow::kpop_prompt_store;
    let _ = crate::cli::kpop_flow::prepare_kpop_run;
    let _ = crate::artifacts::RunArtifacts::exp_log_path;
    let _: Option<crate::cli::kpop_flow::KpopAcpMultiturnCtx> = None;
    let _ = crate::cli::kpop_flow::kpop_emit_startup;
    let _ = crate::cli::kpop_flow::kpop_learn_bundle;
    let _: Option<crate::KpopTurnPrompts> = None;
}

#[test]
fn legacy_timing_error_order_masks_acp_when_both_fail() {
    let acp: Result<(), String> = Err("acp".into());
    let timing: std::io::Result<()> = Err(std::io::Error::other("timing"));
    let legacy = (|| {
        timing.map_err(|e| e.to_string())?;
        acp
    })();
    assert!(
        legacy.unwrap_err().contains("timing"),
        "legacy order should surface timing error, masking ACP (H1)"
    );
}

#[test]
fn merge_acp_prefers_acp_error_when_both_fail() {
    let timing: std::io::Result<()> = Err(std::io::Error::other("timing"));
    let merged = crate::acp_post_run::merge_acp_and_timing_results(Err("acp".into()), timing);
    assert_eq!(merged, Err("acp".into()));
}

#[cfg(test)]
fn kpop_markdown_fixture_context() -> HashMap<String, String> {
    [
        ("plan_path", "./_malvin/run42/plan.md"),
        ("kpop_log_dir", "./_malvin/run42/_kpop"),
        ("review_path", "./review.md"),
        ("result_path", "./_malvin/run42/result.md"),
        ("exp_log", "_malvin/run42/_kpop/exp_log_run42.md"),
        ("malvin_command", "kpop"),
        ("quality_gates", ""),
        ("quality_gates_log", "./_malvin/run42/quality_gates.log"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

#[cfg(test)]
fn assert_substrings_monotonic(prompt: &str, markers: &[&str]) {
    let mut prev: Option<usize> = None;
    for m in markers {
        let pos = prompt
            .find(*m)
            .unwrap_or_else(|| panic!("missing marker substring: {m}"));
        if let Some(p) = prev {
            assert!(p < pos, "marker order prev={p} cur={pos} ({m})");
        }
        prev = Some(pos);
    }
}

#[cfg(test)]
fn assert_prompt_contains_each(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(*needle),
            "missing substring {needle:?}"
        );
    }
}

#[test]
fn kpop_turn_prompts_include_kpop_common_and_exp_log() {
    let tmp = tempfile::tempdir().unwrap();
    let store = PromptStore::with_root(tmp.path().to_path_buf());
    store.ensure_defaults().unwrap();
    let base = kpop_markdown_fixture_context();
    let mut turn = crate::KpopTurnPrompts {
        store: &store,
        base: &base,
        request_text: "do the thing",
        prepend_rules_once: true,
    };
    let kpop = turn.kpop_block(2, 10).unwrap();
    assert_substrings_monotonic(
        &kpop,
        &[
            "AFTER EVERY REQUEST",
            "# Definition: KPop",
            "# This KPOP turn",
        ],
    );
    assert_prompt_contains_each(
        &kpop,
        &[
            "Restate the problem clearly",
            "Hypothesize",
            "_malvin/run42/_kpop/exp_log_run42.md",
        ],
    );
    let mbc2 = turn.mbc2_pure().unwrap();
    assert!(
        !mbc2.contains("AFTER EVERY REQUEST"),
        "mbc2 should not include header/coding rules"
    );
    assert_substrings_monotonic(&mbc2, &["# Definition: KPop", "# Pure MBC2 turn"]);
    assert_prompt_contains_each(
        &mbc2,
        &[
            "Restate the problem clearly",
            "_malvin/run42/_kpop/exp_log_run42.md",
        ],
    );
}
