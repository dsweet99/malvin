# Incomplete handoff

## Done
- Read requirements from the referenced plan: only router prompts or Codex integration may change; fast-task language must not be added to router prompts.
- Updated `src/python/fast_task.py` Codex Docker integration:
  - Mount the complete host Codex package at `/opt/malvin/codex`, not only `bin/codex.js`, so its optional native package is available.
  - Set `MALVIN_CODEX=/opt/malvin/codex/bin/codex.js`.
  - Set Codex PATH to `/opt/malvin:<toolchain>`; the prior path incorrectly contained the executable `/opt/malvin/node` instead of its directory and was overridden by a later PATH assignment.
- Decisive observations:
  - Pi invocation passed FT-01 with reward 1.
  - First Codex attempt failed because `node` was absent from effective PATH.
  - A minimal Docker probe showed `/opt/malvin/node` executes and confirmed PATH must contain `/opt/malvin`.
  - Next Codex attempt reached the launcher, which then reported the optional package missing; mounting the complete package resolved that startup error.
  - Final Codex attempt reached Codex app-server and repeatedly received `401 Unauthorized` from `wss://api.openai.com/v1/responses`; it timed out without grading.
- Quality gates run: `ruff check` passed; `kiss check` passed.

## Remains
- Commit the current `src/python/fast_task.py` change.
- Run the remaining named checks sequentially: `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`, `pytest tests`, and `./admin/malvin_rust_test_gate.sh`.
- If credentials become valid, rerun `./ops/fast_task.py solve --model=codex:gpt-5.6-luna FT-01`; otherwise report the exact 401 blocker. Do not claim Codex fast-task success.
- Inspect `git diff --check` and final status. Avoid committing unrelated pre-existing untracked files.

## Starting position
- Working tree has one intended tracked modification: `src/python/fast_task.py`.
- No router prompt files were changed.
- Commit only that tracked file, using a concise integration-fix message.
