use std::collections::HashMap;

use super::plan_flow_pipeline::{
    commit_plan_prompt_1a, commit_plan_prompt_1b, commit_plan_prompt_2, commit_plan_prompt_3,
};
use super::plan_flow_test_helpers::{
    plan_flow_test_prep, post_1a_content, post_1b_content, post_2_content, test_plan_run_prep,
    test_plan_run_prep_for_plan,
};
use super::{
    prepare_source_plan, resolve_plan_source_path, validate_plan_markers_before_run, PlanArgs,
};
use crate::artifacts::{
    detect_rerun_user_span_end, read_plan_metadata, snapshot_plan_artifact, validate_post_1b,
    validate_post_2, write_plan_metadata, PlanRunMetadata,
};

#[test]
fn resolve_plan_source_path_requires_existing_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "x").expect("write");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let got = resolve_plan_source_path("plan.md").expect("resolve");
    std::env::set_current_dir(old).expect("restore");
    assert!(got.ends_with("plan.md"));
    assert!(resolve_plan_source_path("missing.md").is_err());
    assert!(resolve_plan_source_path("bad path.md").is_err());
}

#[test]
fn ambiguous_markers_without_clean_rerun_fail_validation() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(
        &plan,
        "BEGIN_MALVIN in user\n\n---\nBEGIN_MALVIN\n## Restatement\n",
    )
    .expect("write");
    assert!(validate_plan_markers_before_run(&plan).is_err());
}

#[test]
fn rerun_truncates_machine_block_before_prompts() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n\n---\nBEGIN_MALVIN\nold\n").expect("write");
    prepare_source_plan(&plan).expect("prep");
    assert_eq!(std::fs::read_to_string(&plan).expect("read"), "# User\n\n");
}

#[test]
fn metadata_written_after_1a_shape() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let meta = PlanRunMetadata {
        user_span_end: 10,
        user_span_sha256: None,
    };
    write_plan_metadata(&run_dir, &meta).expect("write");
    let loaded = read_plan_metadata(&run_dir).expect("read").expect("meta");
    assert_eq!(loaded.user_span_end, 10);
}

#[test]
fn post_1b_file_shape_includes_numbered_open_questions() {
    let content = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. First?\n2. Second?\n";
    validate_post_1b(content).expect("valid");
}

#[test]
fn run_dir_snapshots_paths() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "# User\n\n---\nBEGIN_MALVIN\n## Restatement\n").expect("write");
    snapshot_plan_artifact(&run_dir, "plan.p1a.md", &plan).expect("snap");
    assert!(run_dir.join("plan.p1a.md").is_file());
}

#[test]
fn plan_args_debug() {
    let args = PlanArgs {
        plan_path: "plan.md".to_string(),
    };
    let dbg = format!("{args:?}");
    assert!(dbg.contains("plan.md"));
}

#[test]
fn validate_post_2_requires_decisions_after_open_questions() {
    let ok = "# Plan\n\n---\nBEGIN_MALVIN\n## Restatement\nr\n\n## Critique\nc\n\n## Open questions\n1. q\n\n## DECISIONS\n0. **Verdict:** none **Evidence:** n/a\n";
    validate_post_2(ok).expect("valid");
}

#[test]
fn detect_rerun_user_span_end_from_clean_file() {
    let content = "# User\n\n---\nBEGIN_MALVIN\nx\n";
    assert_eq!(detect_rerun_user_span_end(content).expect("ok"), Some(8));
}

#[test]
fn commit_plan_prompt_1a_writes_metadata_and_snapshot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, plan) = plan_flow_test_prep(&tmp);
    let user = "# User\n";
    std::fs::write(&plan, post_1a_content(user)).expect("write");
    let prep = test_plan_run_prep_for_plan(&tmp, &artifacts, &plan);
    let content = std::fs::read_to_string(&plan).expect("read");
    let span = commit_plan_prompt_1a(&prep, &content).expect("commit 1a");
    let expected = detect_rerun_user_span_end(&content).expect("detect").expect("span");
    assert_eq!(span, expected);
    assert!(artifacts.run_dir.join("plan.p1a.md").is_file());
}

#[test]
fn commit_plan_prompt_1b_snapshots_post_critique_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, plan) = plan_flow_test_prep(&tmp);
    std::fs::write(&plan, post_1b_content("# User\n")).expect("write");
    let prep = test_plan_run_prep_for_plan(&tmp, &artifacts, &plan);
    commit_plan_prompt_1b(&prep, &std::fs::read_to_string(&plan).expect("read")).expect("commit");
    assert!(artifacts.run_dir.join("plan.p1b.md").is_file());
}

#[test]
fn commit_plan_prompt_2_writes_decisions_artifact() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, plan) = plan_flow_test_prep(&tmp);
    std::fs::write(&plan, post_2_content("# User\n")).expect("write");
    let prep = test_plan_run_prep_for_plan(&tmp, &artifacts, &plan);
    commit_plan_prompt_2(&prep, &std::fs::read_to_string(&plan).expect("read")).expect("commit");
    let decisions = std::fs::read_to_string(artifacts.run_dir.join("plan.p2.decisions.md"))
        .expect("read decisions");
    assert!(decisions.contains("## DECISIONS"));
}

#[test]
fn commit_plan_prompt_3_splices_fenced_response() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, plan) = plan_flow_test_prep(&tmp);
    let user = "# User\n";
    std::fs::write(&plan, user).expect("write");
    let prep = test_plan_run_prep_for_plan(&tmp, &artifacts, &plan);
    commit_plan_prompt_3(&prep, user.len(), "```markdown\n# Revised\n\nShip.\n```").expect("commit");
    let out = std::fs::read_to_string(&plan).expect("read");
    assert!(out.contains("# Revised"));
    assert!(out.contains("---\nBEGIN_MALVIN"));
}

#[test]
fn commit_plan_prompt_3_rejects_empty_fence() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (artifacts, plan) = plan_flow_test_prep(&tmp);
    let prep = test_plan_run_prep(&tmp, &artifacts, &plan, HashMap::new());
    assert!(commit_plan_prompt_3(&prep, 0, "```markdown\n```").is_err());
}
