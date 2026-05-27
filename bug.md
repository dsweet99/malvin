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

---

# Bug: sandbox memory limit is fail-open when RSS/PSS measurement returns `None`

**Status (fixed):** `watch_process_group_memory` fail-closes after `MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES` (3) consecutive `None` RSS samples and terminates the sandbox. Regression: `watch_process_group_memory_fail_closed_when_rss_unavailable`.

## Severity

High. Malvin advertises a sandbox memory cap in `current_state` and `.malvin/config.toml` (`mem_limit_gb`), but the enforcement loop **never kills** the agent when it cannot obtain a byte total. A run can exceed the configured limit for the whole session while malvin reports success.

## Summary

On Unix, `spawn_process_group_memory_watcher` polls `malvin_session_rss_bytes` every 500ms and terminates the agent process group only when `rss > limit_bytes` inside `if let Some(rss)`. When `malvin_session_rss_bytes` returns `None`, the watcher sleeps and continues — **unknown is treated as under limit**, not as fail-closed.

`malvin_session_rss_bytes` delegates to `pids_sandbox_bytes`, which returns `None` when **no** monitored pid yields readable `/proc` data (all `smaps_rollup` / `status` reads fail or pids disappear). There is no fallback kill, no user-visible warning, and no test requiring termination on measurement failure.

## Affected code

1. **Watcher only acts on `Some(rss)`** — no `else` branch for measurement failure:

```58:76:src/acp/process_group_mem_watch.rs
        if let Some(rss) =
            crate::malvin_sandbox::malvin_session_rss_bytes(Some(pgid), &spawn_pid_baseline)
        {
            if rss > limit_bytes {
                warn!(
                    rss_bytes = rss,
                    limit_bytes,
                    pgid,
                    "malvin sandbox exceeded memory limit; terminating"
                );
                crate::acp::unix_process_group_teardown::terminate_agent_process_group(
                    Some(pgid),
                    &spawn_pid_baseline,
                )
                .await;
                return;
            }
        }
        tokio::time::sleep(POLL_INTERVAL).await;
```

2. **`pids_sandbox_bytes` returns `None` when every pid query fails** (`saw` stays false):

```5:26:src/process_group_rss/linux.rs
pub(in crate::process_group_rss) fn linux_pids_sandbox_bytes(pids: &HashSet<u32>) -> Option<u64> {
    linux_pids_pss_bytes(pids).or_else(|| linux_pids_rss_bytes(pids))
}
// ...
    saw.then_some(total)
```

3. **Empty pid set is treated as 0 bytes (under limit)** — distinct from `None`, but same fail-open spirit:

```59:62:src/process_group_rss/mod.rs
pub fn pids_sandbox_bytes(pids: &HashSet<u32>) -> Option<u64> {
    if pids.is_empty() {
        return Some(0);
    }
```

4. **User-facing cap is still advertised** via `load_mem_limit_bytes` / `format_current_state` even when enforcement is blind.

## Minimal reproduction (logic)

1. Configure a low `mem_limit_gb` in `.malvin/config.toml`.
2. Run an agent session where `sandbox_monitor_pids` is non-empty but every `/proc/{pid}/smaps_rollup` and `/proc/{pid}/status` read fails (permissions, race right after fork, or mocked I/O errors in a unit test).
3. Let the agent allocate memory aggressively.

**Expected (fail-closed):** terminate the sandbox, or surface a hard error that the limit cannot be enforced.

**Actual:** watcher loop never enters the `rss > limit_bytes` branch; session continues until external OOM or normal completion.

## Impact

- **False sense of safety:** Operators and prompts rely on “Sandbox memory: limit N GiB” in `current_state`.
- **Host OOM risk:** Unbounded agent memory while malvin exits 0.
- **Nondeterministic enforcement:** Transient `/proc` failures (load, namespaces, short-lived pids) create windows with no cap.

## Suggested fix direction (not implemented here)

- Treat `None` RSS as over-limit after brief retries, or fail the run with “cannot enforce mem_limit_gb”.
- Log at `warn!` on each consecutive `None` sample; metric for enforcement blind spots.
- Add a unit/integration test that forces `malvin_session_rss_bytes` → `None` and asserts teardown or workflow failure.

---

# Bug: `check_abort` ignores `ABORT:` when `result.md` cannot be read

**Status (fixed):** `check_abort` returns `Err` on unreadable `result.md` (not `None`); merge and `fail_on_abort_for_artifacts` surface `cannot read result file for ABORT check`. Regression: `check_abort_returns_err_when_result_unreadable`.

## Severity

High. Prompts tell the agent to write `ABORT: …` to `{{ result_path }}` (artifact `result.md`) when it cannot proceed honestly. If that file exists but is unreadable, malvin treats the run as **non-abort** and every post-session merge path that calls `check_abort` may return success when the primary ACP result is `Ok(())`.

## Summary

`check_abort` loads `result.md` with `std::fs::read_to_string(result_path).ok()?` and scans lines for `ABORT:`. Any read error (permissions, transient I/O, etc.) becomes `None`, same as a missing file or a file with no abort line. There is no distinction between “no abort signal” and “could not read abort signal.”

After gate-kpop, `do`, `ideas`, `kpop`, and `init` flows call `merge_acp_with_workspace_session_restore_and_check_abort`, which only fails the merge when `check_abort` returns `Some`. A successful ACP session plus an unreadable abort file therefore exits as success.

## Affected code

1. **Read errors become “no abort”:**

```23:31:src/orchestrator/helpers.rs
pub fn check_abort(result_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(result_path).ok()?;
    let text = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("ABORT:") {
            return Some(rest.trim_start().to_string());
        }
    }
    None
}
```

2. **Post-session merge only honors `Some` abort:**

```53:74:src/run_timing/acp_post_run.rs
pub fn merge_acp_with_workspace_session_restore_and_check_abort(
    primary: Result<(), String>,
    work_dir: &Path,
    session_dotfile_backups: &SessionDotfileBackups,
    result_path: &Path,
) -> Result<(), String> {
    let merge_result =
        merge_acp_with_workspace_session_restore(primary, work_dir, session_dotfile_backups);
    if let Some(abort) = crate::orchestrator::check_abort(result_path) {
        return match merge_result {
            Ok(()) => Err(format!("ABORT: {abort}")),
            // ...
        };
    }
    merge_result
}
```

3. **Orchestrator abort gate uses the same helper** via `fail_on_abort_for_artifacts` → `check_abort(&artifacts.artifact_result_md())`.

4. **Prompts bind abort to artifact path** (`workflow_context` inserts `result_path` → `artifact_result_md()`), so the intended file is the run log `result.md`, not workspace root.

## Minimal reproduction (logic)

1. Create `.malvin/logs/<run>/result.md` containing `ABORT: cannot proceed\n`.
2. Remove read permission for the operator (e.g. `chmod 000` on that file).
3. Call `check_abort` on that path.

**Expected (fail-closed):** return `Some("cannot proceed")`, or a hard error that abort cannot be verified.

**Actual:** `read_to_string` fails → `check_abort` returns `None` → merge returns `Ok(())` when ACP succeeded.

## Impact

- **Silent violation of agent stop protocol:** The agent explicitly refused to invent work; malvin reports success.
- **Downstream automation:** CI or scripts that only check malvin exit status miss the abort.
- **Distinct from missing file:** A missing `result.md` correctly yields `None`; an unreadable file with `ABORT:` is indistinguishable from “no abort”.

## Suggested fix direction (not implemented here)

- Map `read_to_string` errors to `Err` (or `Some` with a synthetic abort) instead of `None`.
- When the file exists but is not readable, fail the merge with “cannot read result_path for ABORT”.
- Add a unit test: write `ABORT:` then `chmod 000`; assert `check_abort` or merge does not return success.

