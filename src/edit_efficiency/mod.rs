//! Edit efficiency metrics for agent runs: gross/net byte cost and efficiency ratio.
//!
//! Uses a temporary git index ([`GIT_INDEX_FILE`](https://git-scm.com/docs/git#Documentation/git.txt-codeGITINDEXFILEcode))
//! so the real staging area is untouched. Snapshots use `git add -A` and `git write-tree`;
//! diffs use rename-aware [`git diff`](https://git-scm.com/docs/git-diff).
//!
//! **Orchestrator cadence:** call sites (e.g. `malvin`’s workflow) invoke [`EditEfficiencyMeter::checkpoint`]
//! at prompt boundaries (after each coder prompt and reviewer/kpop pair). Work done *inside* a long agent
//! turn is folded into the next snapshot—finer “every tool step” sampling would need instrumentation inside
//! the agent/ACP layer, not only here.

pub(crate) mod byte_cost;
mod error;
pub(crate) mod git_tree;
pub(crate) mod report;
pub(crate) mod tree_diff;

pub use error::EditEfficiencyError;
pub use report::{finish_and_write_report, maybe_checkpoint};

use std::path::{Path, PathBuf};

/// `net / gross` for reporting; may lose ulp precision for very large totals (display metric only).
#[allow(clippy::cast_precision_loss)]
fn efficiency_ratio(net_bytes: u64, gross_bytes: u64) -> f64 {
    if gross_bytes == 0 {
        1.0
    } else {
        (net_bytes as f64) / (gross_bytes as f64)
    }
}

use crate::edit_efficiency::git_tree::{validate_repo_root, write_tree_from_worktree};
use crate::edit_efficiency::tree_diff::filtered_tree_diff_cost;

/// Result of [`EditEfficiencyMeter::finish`].
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct EditEfficiencyReport {
    /// Sum of filtered byte costs between consecutive snapshots.
    pub gross_bytes: u64,
    /// Filtered byte cost from the initial snapshot to the final tree.
    pub net_bytes: u64,
    /// `net_bytes / gross_bytes`, or `1.0` when `gross_bytes == 0`.
    pub efficiency: f64,
    /// Successful [`EditEfficiencyMeter::checkpoint`] invocations.
    pub checkpoint_calls: usize,
    /// Tree-to-tree diffs summed into [`Self::gross_bytes`]: `checkpoint_calls` plus one when
    /// [`EditEfficiencyMeter::finish`] applied a trailing diff without a matching prior checkpoint.
    pub gross_diff_steps: usize,
}

/// Measures gross/net edit cost using git tree snapshots (no commits).
pub struct EditEfficiencyMeter {
    /// Holds a temp directory; index path is `…/index` and must not exist until git creates it.
    _index_dir: tempfile::TempDir,
    repo_root: PathBuf,
    index_path: PathBuf,
    initial_tree: String,
    baseline_tree: String,
    gross_bytes: u64,
    checkpoint_calls: usize,
    gross_diff_steps: usize,
}

impl EditEfficiencyMeter {
    /// New meter: snapshots the current working tree as `S0` (temp index only).
    ///
    /// # Errors
    ///
    /// Returns an error if `repo_root` is not a git work tree or git commands fail.
    pub fn new(repo_root: impl AsRef<Path>) -> Result<Self, EditEfficiencyError> {
        let repo_root = repo_root.as_ref().to_path_buf();
        validate_repo_root(&repo_root)?;
        let index_dir = tempfile::Builder::new()
            .prefix("malvin_edit_idx_")
            .tempdir()
            .map_err(|e| EditEfficiencyError::GitCommand {
                context: "create temp dir for git index".to_string(),
                source: e,
            })?;
        let index_path = index_dir.path().join("index");
        let initial_tree = write_tree_from_worktree(&repo_root, &index_path)?;
        Ok(Self {
            _index_dir: index_dir,
            repo_root,
            index_path,
            initial_tree: initial_tree.clone(),
            baseline_tree: initial_tree,
            gross_bytes: 0,
            checkpoint_calls: 0,
            gross_diff_steps: 0,
        })
    }

    /// Record a checkpoint: add filtered diff cost since the last snapshot to `gross_bytes`.
    ///
    /// # Errors
    ///
    /// Returns an error if snapshotting or diffing fails.
    pub fn checkpoint(&mut self) -> Result<(), EditEfficiencyError> {
        let new_tree = write_tree_from_worktree(&self.repo_root, &self.index_path)?;
        let delta =
            filtered_tree_diff_cost(&self.repo_root, &self.baseline_tree, &new_tree)?;
        self.gross_bytes += delta;
        self.baseline_tree = new_tree;
        self.checkpoint_calls += 1;
        self.gross_diff_steps += 1;
        Ok(())
    }

    /// Finalize: includes any uncheckpointed working-tree changes in gross, then computes net.
    ///
    /// # Errors
    ///
    /// Returns an error if git commands fail.
    pub fn finish(mut self) -> Result<EditEfficiencyReport, EditEfficiencyError> {
        let final_tree = write_tree_from_worktree(&self.repo_root, &self.index_path)?;
        if final_tree != self.baseline_tree {
            let tail = filtered_tree_diff_cost(
                &self.repo_root,
                &self.baseline_tree,
                &final_tree,
            )?;
            self.gross_bytes += tail;
            self.baseline_tree = final_tree;
            self.gross_diff_steps += 1;
        }
        let net_bytes =
            filtered_tree_diff_cost(&self.repo_root, &self.initial_tree, &self.baseline_tree)?;
        let efficiency = efficiency_ratio(net_bytes, self.gross_bytes);
        Ok(EditEfficiencyReport {
            gross_bytes: self.gross_bytes,
            net_bytes,
            efficiency,
            checkpoint_calls: self.checkpoint_calls,
            gross_diff_steps: self.gross_diff_steps,
        })
    }
}

#[cfg(test)]
mod kiss_stringify_meter {
    #[test]
    fn kiss_stringify_efficiency_ratio() {
        let _ = stringify!(super::efficiency_ratio);
    }
}

#[cfg(test)]
mod meter_tests;
