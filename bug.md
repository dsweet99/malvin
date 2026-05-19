# Bug: false LGTM when fan-out artifact review is empty but workspace `review.md` says LGTM

**Status (fixed):** Fan-out/tidy LGTM uses artifact-only reads (`review_attempt_is_lgtm` → `read_artifact_review_for_fanout_attempt`). `sync_review_file_for_attempt` no longer copies workspace `LGTM` into an empty artifact. `ensure_artifact_review_after_review_write` retries when `review_write` omits the artifact.

## Severity

High. The review loop can treat an attempt as **LGTM** and exit (or run post-LGTM gates in `malvin tidy`) even though **`review_write` never produced a valid artifact review** and fan-out aggregation did not run to completion.

## Summary

The new fan-out review pipeline clears both review files, runs parallel reviewers, then runs `review_write` to aggregate into the **artifact** path `{{ review_path }}` (`_malvin/<run>/review.md`). LGTM was decided by `review_attempt_is_lgtm` → `sync_review_file_for_attempt` (since fixed: artifact-only).

When the artifact review is **missing or whitespace-only**, `sync_review_file_for_attempt` **copies whatever is in workspace `./review.md` into the artifact** and returns that text. If workspace contains `LGTM`, the attempt is scored as LGTM.

There is **no post-`review_write` check** that the artifact review file exists and is non-empty (unlike `check_plan`, which retries when the agent omits the review file). ACP success for `review_write` is therefore not evidence that aggregation wrote `review_path`.

After `run_review_fanout_prefix` clears both files, workspace `./review.md` can be recreated before `review_attempt_is_lgtm` runs by:

- A parallel fan-out reviewer writing to the wrong path (same `cwd`, full coding rules prepended, no hard prohibition on touching `./review.md`).
- `review_write` writing LGTM to `./review.md` instead of the artifact path (agent error).
- Any other coder step that leaves workspace `LGTM` between clear and sync (less common).

The artifact-preferring branch of sync **does not help** when the artifact is still empty: stale workspace `LGTM` is promoted into the artifact and then honored.

## Affected code

1. **Clear at attempt start** — both reviews removed:

```48:51:src/orchestrator/review_attempt_kernel.rs
    clear_review_file(&artifact_review)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;
```

2. **Workspace fallback when artifact empty** — copies workspace text (including `LGTM`) into the artifact:

```53:75:src/review_sync/attempt.rs
    if let Some(artifact_text) = read_nonempty_review(artifact_review_path, "artifact")? {
        return Ok(Some(artifact_text));
    }

    if workspace_review_path.exists() {
        let workspace_text = std::fs::read_to_string(workspace_review_path).map_err(|e| {
            // ...
        })?;
        if workspace_text.trim().is_empty() {
            clear_artifact_review_to_empty(artifact_review_path)?;
            return Ok(None);
        }
        ensure_parent_dir(artifact_review_path)?;
        std::fs::write(artifact_review_path, &workspace_text).map_err(|e| {
            // ...
        })?;
        return Ok(Some(workspace_text));
    }
```

3. **LGTM decision after `review_write`** — no verification that `review_write` wrote the artifact:

```51:62:src/orchestrator/review_loop.rs
    let reviewers_subdir = run_review_fanout_prefix(&*orchestrator.client, &kernel).await?;
    run_review_write_coder_session(ReviewWriteCoderSession {
        // ...
    })
    .await?;
    let lgtm = review_attempt_is_lgtm(orchestrator.artifacts)?;
```

4. **Contrast: `check_plan` retries when review file missing** — fan-out / `review_write` have reviewer preflight only, not final-review preflight:

```17:40:src/orchestrator/check_plan.rs
    for attempt in 0..max_loops {
        // ...
        let Some(contents) = run_check_plan_attempt(orchestrator, context, &review_path).await?
        else {
            // retry — agent did not write review file
            continue;
        };
        // ...
    }
    Err(WorkflowError(
        "check_plan: agent did not write review file after retries".to_string(),
    ))
```

5. **Parallel fan-out shares one workspace `cwd`** — reviewers use `ReviewerRestorePolicy::NoRestore` and `sync_workspace_review: false`, but all jobs in a chunk share `work_dir`; nothing prevents a reviewer from writing `./review.md`.

## Minimal reproduction (logic)

After the fan-out prelude clears reviews:

1. Do **not** create `_malvin/<run>/review.md` (simulate `review_write` omitting the artifact).
2. Create `./review.md` with exactly `LGTM\n`.
3. Call `review_attempt_is_lgtm`.

**Expected (safe behavior):** non-LGTM or hard error (“review_write did not write artifact review”).

**Actual:** `sync_review_file_for_attempt` copies workspace `LGTM` into the artifact; `is_lgtm_str` returns true.

Existing unit test `sync_review_file_for_attempt_writes_workspace_text_to_artifact` encodes step 2–3 as **intended** fallback behavior; the bug is using that fallback **after** fan-out + `review_write` without ensuring the artifact was written.

## Impact

- **`malvin code`:** Review loop can exit on false LGTM; `concerns.md` is skipped; implementation may ship without aggregated reviewer findings.
- **`malvin tidy`:** May declare LGTM and run gates; gate failure is a partial backstop, but reviewer findings are still lost and behavior is nondeterministic under parallel reviewers.
- **Downstream prompts** that read `{{ review_path }}` (artifact) may see materialized false `LGTM` after sync.

## Suggested fix direction (not implemented here)

- After `review_write`, require non-empty artifact `review.md` (retry or fail like `check_plan`).
- For fan-out attempts, do **not** fall back to workspace `review.md` for LGTM (or re-clear workspace review after fan-out, before `review_write`).
- Optionally drop `workspace_review_path` from fan-out `ReviewerPromptPair` entirely so reviewers are not anchored to the legacy workspace review file.
