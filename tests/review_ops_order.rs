//! Regression: syncs after the review prompt and skips kpop when `review.md` is already `LGTM`.

#[test]
fn reviewer_ops_syncs_and_checks_lgtm_before_kpop_prompt() {
    let ops = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/ops_body.inc"));
    let sync = ops
        .find("sync_review_file(pair.workspace_review_path, pair.artifact_review_path)")
        .expect("expected sync from workspace review.md to run artifact after review prompt");
    let branch = ops
        .find("if is_lgtm(pair.artifact_review_path)")
        .expect("expected LGTM check on artifact before optional kpop");
    let kpop = ops
        .find("s.prompt(pair.kpop_body, pair.kpop_log)")
        .expect("expected kpop session/prompt after conditional branch");
    assert!(sync < branch && branch < kpop);
}
