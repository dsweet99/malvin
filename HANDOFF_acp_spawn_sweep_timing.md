# INCOMPLETE HANDOFF — acp spawn sweep test timing

Status: incomplete. No production code was changed in this turn.

## Done
- Audited the vague plan at `/home/dsweet/.malvin_home/logs/eb7ef333a92a6d41/20260818_191345_92f8v4s6/plan_skiyx.md`; it contains only `Pls finish up the last task.`
- Ran the required gates before this turn's test edit: `ruff check`, `kiss check`, Clippy, `pytest tests` (156 passed), and the Rust gate (796 + 817 passed).
- Identified the direct `VISION.md` violation: `malvin_doc_does_not_sweep_but_models_does` measured 1.868s in the full gate and 1.554s isolated.
- Applied a test-only change in `tests/malvin_acp_spawn_sweep_contract.rs`:
  - replaced `fresh_workdir` with `tempfile::tempdir` plus the required `git init` precondition;
  - split the `--doc` assertion into `malvin_doc_does_not_sweep_stale_locks`, preserving both behavioral assertions.
- The modified test binary compiled and all four tests passed, but the target test still measured 1.533s isolated in one run and therefore remains above the 1.5s constraint.

## What remains
- Optimize `malvin_doc_does_not_sweep_but_models_does` below 1.5s without changing gate thresholds, skipping coverage, or weakening the sweep contract.
- A promising next experiment is to invoke `models pi:` instead of `models` and remove the now-unnecessary fake `agent` PATH setup, because the test only checks entrypoint sweep behavior; verify that this still reaches the production sweep and that the stale lock is removed.
- Rerun the focused test, then all required gates sequentially after the final edit.
- Audit the final tree and preserve unrelated existing modifications/untracked files.

## Starting position
- Current commit should contain only the test edit and this envelope.
- Existing unrelated dirty files remain modified/untracked, including `src/cli/models_cmd_cursor.rs` and `src/cursor_sdk/node_resolve.rs`.
- The target file is `tests/malvin_acp_spawn_sweep_contract.rs`.
- Last observed focused result before handoff: target passed in 1.533s; full target binary passed all 4 tests in 1.636s.
