# Incomplete handoff: pi::sdk in-process — GATES GREEN; Phase 5 deletion sweep remains

Status: **all 5 gates PASS this turn** — ruff ✓ kiss ✓ clippy ✓ pytest 157 ✓
rust gate **859/859** ✓. One code edit made (see below). No `--git` (standing
instruction; harness posts the `ok` commits). Work dir `/home/dsweet/Projects/malvin`.

## Done this turn

1. **Root-caused and fixed the failing auth-filter test**
   (`src/cli/models_cmd_auth_filter_tests.rs`). Prior turn proved
   `pi:llamacpp/` listing rows are impossible on crates.io `pi_agent_rust`
   0.1.23 (`ad_hoc_model_entry` is per-request only; legacy catalog + upstream
   snapshot contain no llamacpp). Also solved the 02:04 mystery: `git log
   --follow` shows the test originally drove a FAKE `pi` binary whose table
   contained `llamacpp local`; the crate-path rewrite kept the assertion but
   dropped its data source. New assertion set keeps the test's intent with real
   registry rows: openai hidden without key / openrouter shown with key /
   **zhipuai** (unmapped provider) stays visible + `is_provider_authenticated(
   "llamacpp") == true` at the auth layer. Both module tests pass.

## Remains (Phase 5 deletion sweep — plan mandates, in this order)

Everything below is verified reachable-or-dead as noted; no behavior change is
intended, deletions only. After each step run `cargo test --lib pi_sdk::` fast,
full gates at the end.

1. **`MALVIN_PI` / binary resolver (Phase 4 explicit: "Delete
   `resolve_pi_bin` / `MALVIN_PI` / version check once nothing calls them")**
   - Live `--do` + `malvin models pi:` passed without the binary (prior turn,
     exp_log g1-retry2), so the precondition is met.
   - `src/pi_sdk/discover.rs`: delete `resolve_pi_bin`,
     `pi_missing_binary_message`, `PI_MISSING_HINT`, `pi_path_is_executable`,
     and the `#[cfg(test)]` version helpers (`pi_version_ok`,
     `parse_pi_version`, `PI_MIN_VERSION`, `parse_semver_triple`,
     `leading_u32`) + their tests. Keep the file only if something else needs
     it, else delete file + `mod discover;`.
   - `src/pi_sdk/mod.rs:20`: drop `pub use discover::{...}`.
   - `src/pi_sdk/kiss_coverage_tests.rs` + `src/pi_sdk/discover_tests.rs`:
     delete the referencing lines/tests (discover_tests is entirely about the
     resolver — delete file + `mod` line).
   - `src/python/fast_task.py` (= `ops/fast_task.py` thin wrapper): delete
     `ft_resolve_pi_bin` (L96) and `PI_BIN_REMOTE` (L66) plus the
     `MALVIN_PI={PI_BIN_REMOTE}` negative assertions at L1105-1117, L1383-1384,
     and the resolver test block ~L1668-1691. NOTE: keep the *negative*
     docker-mount assertions' spirit if trivial, else delete — plan only
     requires "stop requiring that once the crate is linked", which is already
     true (ft_docker_agent_cmd no longer mounts pi).
2. **RPC fallback path (plan non-goal: "Supporting `--rpc` fallback once the
   in-process path works")**
   - `BridgeWire::PiRpc` is constructed NOWHERE in non-test code (only
     `codex_sdk/session_process.rs:44` builds `CodexRpc`; cursor builds
     `NodeBridge`; `PiEmbeddedSession` bypasses `BridgeSession` entirely) —
     verified via grep this turn. So the arm at
     `src/bridge_sdk/session.rs:83,95` is unreachable dead code.
   - Delete: `src/pi_sdk/session_io.rs` (whole file: `pi_send_prompt`,
     `pi_write_abort`, `pi_send_new_session`, ...), `src/pi_sdk/protocol.rs`
     (keep only if a golden-file test still wants it — plan allows keeping
     `map_event.rs` JSON mapping for old RPC fixtures, but `map_event_tests.rs`
     should be ported or dropped; `map_agent_event_tests.rs` already covers
     the typed mapper), `mod.rs` re-exports `send_prompt`/`write_abort`
     (L32) + `spawn_bridge` stays (that's the embedded one), the
     `BridgeWire::PiRpc` enum variant + its two match arms in
     `bridge_sdk/session.rs`, and `mock_pi.sh`.
   - `src/pi_sdk/session_spawn.rs` currently exports `pi_spawn_bridge` which
     returns `SdkSession::Pi` (embedded) — that name stays; it is the live path.
3. **Kiss witnesses referencing deleted names**: update
   `kiss_coverage_tests.rs` / `session_spawn_tests.rs` name lists (remove
   `pi_write_line`, `pi_write_abort`, `pi_send_new_session`, `pi_send_prompt`,
   `pi_wait_for_response`, `pi_read_line*`, `pi_drain_until_run_done`,
   `pi_finish_run_done`, `pi_next_req_id`, `PI_REQ_SEQ`, `BridgeWire::PiRpc`,
   discover names) — then `kiss check` to confirm no NEW sub-90% files appear.
4. **Docs**: grep for `MALVIN_PI` / "install pi" / "pi binary" in README.md,
   `default_prompts/docs/{malvin,models}.md`, `--doc` strings — prior handoffs
   say these are already crate-path; verify, fix stragglers.
5. **Full gates**: ruff; kiss; clippy `--jobs 3 --all-targets --all-features
   -- -D warnings -W clippy::cargo`; `pytest tests`;
   `./admin/malvin_rust_test_gate.sh`.
6. **Optional live smoke** (acceptance 1–3): `malvin --model=pi:<prov>/<model>
   --do Hello` and `malvin models pi:` with no `pi` on PATH / no `MALVIN_PI`.

## Epistemic uncertainty for the next agent

- `map_event.rs` (JSON mapper) deletion is safe ONLY if no remaining test or
  golden fixture needs it; `map_event_tests.rs` exists — decide port-vs-drop
  before deleting (plan: "keep JSON mapping only if we still want a
  golden-file test against old RPC fixtures").
- `protocol_tests.rs` pins the wire encode/decode; delete together with
  `protocol.rs` if you go that way.
- `providers_list.rs` `parse_list_providers_table` + tests are already
  `#[cfg(test)]`-only leftovers of `pi --list-providers`; plan Phase 4 says
  replace the binary path (done) — deleting the dead parser + its tests is
  consistent but optional; kiss witness references must follow.

## Next-agent start

Step 1 above (discover.rs deletion), `cargo test --lib pi_sdk::` after each
numbered step, full gates at the end. Predicted: 45–70 min.
