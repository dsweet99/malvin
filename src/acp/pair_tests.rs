use std::path::Path;

use crate::acp::ReviewerPromptPair;

#[test]
fn reviewer_prompt_pair_fields() {
    let _pair = ReviewerPromptPair {
        cwd: Path::new("."),
        workspace_review_path: Path::new("review.md"),
        artifact_review_path: None,
        review_body: "body",
        review_who: "who",
        review_log: Path::new("log"),
        sync_workspace_review: true,
    };
}
