/// Paths and rendered body for one reviewer prompt in a single ACP session.
pub struct ReviewerPromptPair<'a> {
    pub cwd: &'a Path,
    /// Workspace `review.md` (synced to the artifact after the review prompt).
    pub workspace_review_path: &'a Path,
    /// `_malvin/.../review.md` artifact path when [`Self::sync_workspace_review`] is true.
    pub artifact_review_path: Option<&'a Path>,
    pub review_body: &'a str,
    pub review_who: &'a str,
    pub review_log: &'a Path,
    /// When false, the reviewer writes only to paths named in the prompt (fan-out jobs).
    pub sync_workspace_review: bool,
}

#[cfg(test)]
mod pair_tests {
    include!("pair_tests.inc");
}
