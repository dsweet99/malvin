use std::collections::HashMap;
use std::path::PathBuf;

use crate::artifacts::RunArtifacts;
use crate::cli::{KpopArgs, SharedOpts};
use crate::prompts::{PromptError, PromptStore};

pub fn kpop_emit_startup(
    kpop: &KpopArgs,
    shared: &SharedOpts,
    artifacts: &RunArtifacts,
) -> Result<(), String> {
    let request = crate::cli::cli_request::require_cli_request(kpop.request.as_ref(), "kpop")?;
    crate::cli::run_emit::emit_run_startup_sequence(
        artifacts,
        crate::cli::run_emit::RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &request,
    )
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
fn kpop_emit_startup_creates_malvin_run_under_root() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("kpop", Some(tmp.path())).expect("art");
    let shared = crate::cli::SharedOpts {
        model: crate::config::DEFAULT_CLI_MODEL.into(),
        no_force: true,
        no_tee: true,
        no_markdown: true,
        verbose: false,
        doc: false,
    };
    let kpop = crate::cli::KpopArgs {
        max_hypotheses: 1,
        no_learn: true,
        request: Some("smoke".into()),
    };
    kpop_emit_startup(&kpop, &shared, &artifacts).expect("startup");
    let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
    assert!(log.contains("Memory:"));
    assert!(artifacts.run_dir.starts_with(tmp.path().join(".malvin/logs")));
}

#[test]
fn kpop_learn_bundle_none_when_learn_disabled() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts =
        crate::artifacts::create_run_artifacts_from_text("kpop", Some(tmp.path())).expect("art");
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let ctx = HashMap::new();
    assert!(
        kpop_learn_bundle(&store, &ctx, false, &artifacts)
            .expect("bundle")
            .is_none()
    );
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
        ("plan_path", "./.malvin/logs/run42/plan.md"),
        ("kpop_log_dir", "./.malvin/logs/run42/_kpop"),
        ("review_path", "./review.md"),
        ("result_path", "./.malvin/logs/run42/result.md"),
        ("exp_log", ".malvin/logs/run42/_kpop/exp_log_run42.md"),
        ("malvin_command", "kpop"),
        ("quality_gates", ""),
        ("quality_gates_log", "./.malvin/logs/run42/quality_gates.log"),
        ("advice_path", "./.malvin/advice.md"),
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
            "Regular memories",
            "# Definition: KPop",
            "# This KPOP turn",
        ],
    );
    assert_prompt_contains_each(
        &kpop,
        &[
            "Restate the problem clearly",
            "Hypothesize",
            "Complete up to `2` KPOP iterations",
            "iterations budget",
            ".malvin/logs/run42/_kpop/exp_log_run42.md",
            "Do not write KPop logs under repo-root `./_kpop/`",
        ],
    );
    assert!(
        !kpop.contains("Complete exactly"),
        "kpop_block must use up-to wording: {kpop:?}"
    );
    assert!(
        !kpop.contains("remaining_hypotheses"),
        "kpop_block must not reference remaining_hypotheses: {kpop:?}"
    );
    let mbc2 = turn.mbc2_turn().unwrap();
    assert!(
        !mbc2.contains("Regular memories"),
        "mbc2 should not include header/coding rules"
    );
    assert!(mbc2.contains("# MBC2"));
    assert_prompt_contains_each(
        &mbc2,
        &[
            "do the thing",
            "produce exactly one MBC2 hypothesis",
            ".malvin/logs/run42/_kpop/exp_log_run42.md",
        ],
    );
}
