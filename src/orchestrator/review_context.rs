//! Types for one review phase and one review attempt.

use std::collections::HashMap;
use std::path::Path;

/// One review phase: prompt template, UI label, stable id for logs, and template context.
pub struct ReviewPhaseArgs<'a> {
    pub review_prompt: &'a str,
    pub progress_label: &'a str,
    pub phase_id: &'a str,
    pub context: &'a HashMap<String, String>,
}

pub struct ReviewAttemptCtx<'a> {
    pub review_prompt: &'a str,
    pub progress_label: &'a str,
    pub phase_id: &'a str,
    pub attempt: usize,
    pub workspace_review_path: &'a Path,
    pub review_path: &'a Path,
    pub context: &'a HashMap<String, String>,
}
