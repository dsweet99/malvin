/// Paths and rendered bodies for one reviewer `review` + `kpop` pair in a single ACP session.
pub struct ReviewerPromptPair<'a> {
    pub cwd: &'a Path,
    /// Workspace `review.md` (synced to the artifact after the review prompt).
    pub workspace_review_path: &'a Path,
    /// `_malvin/.../review.md` copy used for [`crate::review_sync::is_lgtm`].
    pub artifact_review_path: &'a Path,
    pub review_body: &'a str,
    pub kpop_body: &'a str,
    pub review_who: &'a str,
    pub kpop_who: &'a str,
    pub review_log: &'a Path,
    pub kpop_log: &'a Path,
}
