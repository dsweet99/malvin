# Incomplete handoff envelope

## Done
- Implemented and verified the `codex:` backend path and Codex model discovery surfaces already present in the repository.
- Fixed `clippy::items-after-test-module` in `src/codex_sdk/mod.rs`.
- Fixed `clippy::items-after-statements` in `src/codex_sdk/discover.rs`.
- Added executable-mode tests/references for Codex and Pi discovery.
- Direct results: `ruff check` passed; `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo` passed; `pytest tests` passed (156 tests); `./admin/malvin_rust_test_gate.sh` passed (800 + 831 tests).

## Remaining
- `kiss check` remains failing with static Rust coverage reports: `src/codex_sdk/discover.rs` 75% and `src/pi_sdk/discover.rs` 86%, both naming `path_is_executable`.
- The added runtime test calls did not change kiss's reported percentages. This indicates the remaining issue may be kiss's exact AST/reference matcher rather than runtime coverage; this is a hypothesis requiring inspection of kiss-ai's coverage implementation.
- The final `kiss check` after the last tiny test-reference edit is still unverified.
- No completion sentinel is allowed until `kiss check` passes and all named checks are directly green after the final diff.

## Next starting position
1. Run `git diff --check` and inspect the current diff.
2. Run `kiss check` directly.
3. Inspect kiss-ai 0.4.9 Rust coverage matcher, especially `/home/dsweet/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/kiss-ai-0.4.9/src/rust_test_refs/coverage_map.rs` (copy into the permitted workspace if tool restrictions require it).
4. Determine whether exact symbol names, test module ownership, or file naming controls static coverage. Do not alter `.kissconfig`.
5. Make the smallest matcher-compatible test/reference change, rerun `kiss check`, then rerun all named gates sequentially.
