# Bug: `malvin tidy --max-loops 1` cannot succeed after a non-LGTM review even if concerns fix everything

## Status

**Fixed** in working tree (`run_tidy_post_concerns_recovery` after non-LGTM `--max-loops 1` concerns). Contract: `tests/tidy_max_loops_one_contract.rs`.

## Summary

With `--max-loops 1`, when the reviewer returned non-LGTM, tidy ran one `tidy_concerns` coder turn and then exited with `tidy did not converge within 1 iterations`. It did not re-run quality gates or the review fan-out after that concerns turn, so a successful concerns pass could not produce exit code 0.

## Fix

After `run_tidy_concerns_coder_turn` on the `max_loops == 1` non-LGTM path, `run_tidy_post_concerns_recovery` re-runs gates and one `tidy_review_attempt_with_retries`, returning `Ok(())` when LGTM and gates pass (same tail as bonus gate recovery, without a second concerns pass).

## Related defects (operational, fixed)

`review_prompt_log_path` uses `log_attempt` for the base name on tidy and `malvin code` paths, with a shared inner-retry `_try_N` rule.

Post-concerns recovery on `--max-loops 1` non-LGTM now passes `max_attempts + 1` as the review fan-out log attempt (same as bonus gate recovery), so recovery does not overwrite `reviewers_spawn_attempt_1.log`. Stdout uses `tidy recovery (review attempt N, max-loops M)` instead of `tidy iteration N+1/M`. Inner `review_write` missing-artifact retries per outer iteration use `max_loops.max(1)` again (not a fixed cap of 2). Regressions: `tidy_max_loops_one_non_lgtm_concerns_recovery_can_exit_zero`, `tidy_inner_review_write_retries_allow_at_least_max_loops_per_outer_iteration`.
