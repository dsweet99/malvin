#[cfg(unix)]
mod common;

#[cfg(unix)]
use std::time::Duration;

use common::{
    TidySpawn, acp_mock_tidy_fanout_lgtm_js, bin_path_with_kiss_fail_until_n_passes, only_run_dir,
    seed_git_kiss_cargo_gate_workspace, spawn_tidy_with_timeout, test_home_workspace,
    workspace_kiss_check_only, write_mock_executable,
};

const TIDY_BONUS_GATE_TIMEOUT: Duration = Duration::from_secs(20);

#[cfg_attr(unix, test)]
fn tidy_bonus_review_uses_attempt_index_matching_printed_iteration() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-bonus-attempt-index.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 4);
    let mock = root.path().join("mock-tidy-bonus-attempt-index");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy_with_timeout(
        &TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "3"],
        },
        TIDY_BONUS_GATE_TIMEOUT,
    );
    assert!(
        out.status.success(),
        "expected tidy success after bonus gate recovery: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy recovery (review attempt 4, max-loops 3)"),
        "bonus pass must print recovery banner with log attempt max+1: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let bonus_review_log = run_dir.join("reviewers_spawn_attempt_4.log");
    assert!(
        bonus_review_log.is_file(),
        "bonus review must write reviewers_spawn_attempt_4.log when stdout says tidy recovery \
         (review attempt 4, max-loops 3); \
         got run_dir={run_dir:?}"
    );
}
