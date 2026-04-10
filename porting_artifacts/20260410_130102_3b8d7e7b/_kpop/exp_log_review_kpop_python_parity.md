# KPOP experiment log: review → kpop ordering vs Python `orchestrator.py`

## Problem (restatement)

The Rust port’s reviewer path (`src/agent/ops.rs`) always issued **two** `session/prompt` calls per attempt—review, then kpop—in a single ACP session. Python’s `orchestrator.py` instead runs the **review** prompt, **syncs** workspace `review.md` into the run artifact, checks for **`LGTM`**, and only runs **kpop** when that check fails. That mismatch could waste work and change behavior relative to the reference implementation.

---

## Hypothesize

**Falsifiable explanation:** The regression was caused by **missing** `sync_review_file` + `is_lgtm` on the artifact **between** the review prompt and the kpop prompt, so the code never had a chance to short-circuit after a successful review.

---

## Predict (falsifying test)

**Test:** `tests/review_ops_order.rs` — `reviewer_ops_syncs_and_checks_lgtm_before_kpop_prompt`

If the hypothesis is true (ordering bug in source), either:

- the test file would not exist, or  
- `src/agent/ops.rs` would **not** contain `sync_review_file(pair.workspace_review_path, …)` and `if is_lgtm(pair.artifact_review_path)` **before** `s.prompt(pair.kpop_body, …)`, and the test would **fail**.

If the fix is present, the test should **pass**.

---

## Falsify

**Command:** `cargo test --test review_ops_order -q`

**Result (2026-04-10):** `running 1 test` — **1 passed**, 0 failed.

**Conclusion:** The hypothesis is **not falsified** by this test: the repository state includes the short-circuit between review and kpop, and the structural regression test passes. (A pre-fix tree would have failed the ordering assertion or lacked the new paths on `ReviewerPromptPair`.)

---

## Status

Treated as **resolved** in-tree: shared helpers live in `src/review_sync.rs`; reviewer pair carries workspace + artifact review paths; ops syncs and checks LGTM before optional kpop.
