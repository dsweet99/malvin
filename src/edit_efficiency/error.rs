//! Errors from git snapshotting and diffing for edit-efficiency metrics.

use std::path::PathBuf;

use thiserror::Error;

/// Failure while measuring edit efficiency (git I/O or output parsing).
#[derive(Debug, Error)]
pub enum EditEfficiencyError {
    /// Running a git subprocess failed.
    #[error("git command failed: {context}")]
    GitCommand {
        context: String,
        #[source]
        source: std::io::Error,
    },
    /// Git exited with a non-zero status.
    #[error("git exited with status {status}: {stderr}")]
    GitFailed { status: i32, stderr: String },
    /// Git produced output that was not valid UTF-8 where required.
    #[error("git output was not valid UTF-8: {context}")]
    Utf8 { context: String },
    /// Failed to parse `git diff --name-status -z` records.
    #[error("failed to parse name-status output")]
    ParseNameStatus,
    /// Repository path was invalid for git.
    #[error("invalid repo root: {0}")]
    InvalidRepo(PathBuf),
}
