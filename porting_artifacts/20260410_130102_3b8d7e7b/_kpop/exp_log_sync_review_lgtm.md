# KPOP experiment log: `sync_review_file` vs artifact `LGTM`

## Problem (restated)

After a reviewer step, the orchestrator copies the workspace `review.md` onto `_malvin/<run>/review.md` and then checks that file for an exact `LGTM`. A naive copy overwrote a valid `LGTM` in the run directory whenever the workspace file existed but was empty (or whitespace-only), for example because the model wrote `LGTM` only under `_malvin/.../review.md` and left an empty stub at the repo root. The fix is to **not** copy when the workspace content is empty after trimming.

## Hypothesis (H1)

**H1:** The implementation still contains a path where an empty or whitespace-only workspace overwrites the artifact and removes `LGTM` after the intended fix.

This is falsified if the dedicated regression tests fail under that scenario (they assert the artifact still reads `LGTM` after sync).

## Predict / falsifying test

If **H1** is true, at least one of these should fail:

- `orchestrator::tests::sync_review_file_skips_empty_workspace_so_artifact_lgtm_is_preserved`
- `orchestrator::tests::sync_review_file_skips_whitespace_only_workspace`

If **H1** is false (fix behaves as intended for those cases), both pass.

## Falsify (command + outcome)

```text
cargo test sync_review_file --no-fail-fast
```

**Result (2026-04-10):** Exit code **0**. All three tests passed:

- `sync_review_file_skips_empty_workspace_so_artifact_lgtm_is_preserved` — ok  
- `sync_review_file_skips_whitespace_only_workspace` — ok  
- `sync_review_file_copies_nonempty_workspace_to_artifact` — ok  

**Conclusion:** **H1 is rejected** for the scenarios covered by these tests: empty/whitespace workspace no longer clobbers a pre-existing `LGTM` in the artifact path.

## Follow-up (not falsified here)

A different hypothesis—that **skipping** empty sync can leave a stale `LGTM` when a human clears only the workspace file—is not exercised by these tests and remains a product trade-off, not disproven by this run.
