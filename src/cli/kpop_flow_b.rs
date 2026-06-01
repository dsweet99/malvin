use crate::artifacts::RunArtifacts;
use crate::cli::{KpopArgs, SharedOpts};

#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
use crate::prompts::PromptStore;

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
        max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
        doc: false,
    };
    let kpop = crate::cli::KpopArgs {
        max_loops: 1,
        max_hypotheses: 1,
        tenacious: false,
        request: Some("smoke".into()),
    };
    kpop_emit_startup(&kpop, &shared, &artifacts).expect("startup");
    let log = std::fs::read_to_string(artifacts.run_dir.join("command.log")).expect("log");
    assert!(log.contains("Memory:"));
    assert!(artifacts.run_dir.starts_with(crate::malvin_logs_root(tmp.path())));
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
        ("review_path", "./.malvin/logs/run42/review.md"),
        ("result_path", "./.malvin/logs/run42/result.md"),
        ("exp_log", ".malvin/logs/run42/_kpop/exp_log_run42.md"),
        ("malvin_command", "kpop"),
        ("quality_gates", ""),
        ("quality_gates_log", "./.malvin/logs/run42/quality_gates.log"),
        ("advice_path", "./.malvin/advice.md"),
        ("logs_dir", "./.malvin/logs/run42"),
        (
            "current_state",
            "User: test\nDate/time: now\nSandbox memory: limit 4 GiB\nRetry: not a retry",
        ),
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
}
