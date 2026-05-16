# Bug: false LGTM when fan-out artifact review is empty but workspace `review.md` says LGTM

## Status

**Fixed** on branch `dsweet/gates`.

## What was wrong

After fan-out + `review_write`, `review_attempt_is_lgtm` used `sync_review_file_for_attempt`, which copied workspace `./review.md` (including stale `LGTM`) into the empty artifact and scored LGTM.

## Fix (current behavior)

- `read_artifact_review_for_fanout_attempt`: artifact-only read for fan-out LGTM (no workspace promotion).
- `ensure_artifact_review_after_review_write`: errors when artifact review is missing or whitespace-only after `review_write`.
- `run_code_review_phase`: retries the review attempt within `max_loops` when the artifact is missing (like `check_plan`), instead of aborting the whole review phase.
- Wired in `review_loop.rs` and `tidy_flow/helpers.rs` (via `tidy_review_attempt_with_retries`).
- Regression tests in `tests/cli_parity_code_fanout.rs` and `review_attempt_kernel.rs` unit tests.

## Remaining hardening (non-blocking)

- `sync_review_file_for_attempt` still exists for legacy/tests; fan-out must keep using artifact-only paths.
- Fan-out `ReviewerPromptPair` still accepts `workspace_review_path` though sync is disabled for fan-out jobs.
