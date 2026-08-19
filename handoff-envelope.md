# Incomplete handoff envelope

## Done
- Audited the failing named criterion directly with `kiss check`.
- Inspected kiss-ai 0.4.9 source. Its coverage map matches referenced function names and disambiguates duplicate names using `name_files`; executable test calls are collected from reachable test expressions.
- Tested three candidate acts with distinct axes:
  - Act A: exact unaliased test references; discarded because kiss remained at 75%/86%.
  - Act B: test placement/authority changes; discarded because kiss remained at 75%/86%.
  - Act C: backend-specific helper names; selected because it removes the duplicate-name ambiguity without changing runtime policy or thresholds.
- Applied Act C across Codex and Pi discovery callers and witnesses.
- The first Act C `kiss check` removed the original coverage failures, exposing three structural `calls_per_function` violations in existing Codex functions.

## Remaining
- The current working tree was last mechanically repaired after an over-broad replacement created `codex_codex_path_is_executable` and `pi_pi_path_is_executable`; those doubled names have now been corrected, but compilation has not been rerun after correction.
- Required checks are therefore unresolved for the current diff. In particular, run `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo` first, then `kiss check`.
- `kiss check` currently has structural violations in `src/codex_sdk/session_spawn.rs:codex_spawn_bridge`, `src/codex_sdk/discover.rs:list_codex_models`, and `src/codex_sdk/session_io.rs:codex_send_prompt`, each exceeding the configured calls-per-function threshold. These were exposed only after the coverage issue was fixed.
- Do not edit `.kissconfig` or include unrelated untracked operator artifacts.

## Next-agent starting position
1. Inspect `git diff --check` and `grep -RIn 'codex_path_is_executable\|pi_path_is_executable' src/codex_sdk src/pi_sdk`.
2. Run Clippy to establish compile status.
3. Run `kiss check`; extract helpers from the three named Codex functions, preserving protocol order and behavior.
4. Rerun `kiss check`, then `ruff check`, `cargo clippy`, `pytest tests`, and `./admin/malvin_rust_test_gate.sh` sequentially.
5. Commit only intended tracked changes. The named Done criterion remains unsatisfied until the direct kiss check passes.
