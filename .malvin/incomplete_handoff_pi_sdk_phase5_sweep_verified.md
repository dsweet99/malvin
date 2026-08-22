# Incomplete handoff: pi::sdk in-process — Phase 5 deletion sweep VERIFIED, NOT EXECUTED

Status: **no code edits this turn** (iteration budget consumed by verification).
Baseline unchanged: all 5 gates PASS at g1-retry5. This envelope supersedes
`.malvin/incomplete_handoff_pi_sdk_phase5_cleanup.md` with corrected scope and
one hazard the old envelope got wrong.

Work dir `/home/dsweet/Projects/malvin`. Exp log:
`_run/exp_log_20260822_040428_1z8fhizh_g1.md` (H1–H4 with grep evidence).

## Verified facts (evidence in exp log)

1. Dead-code set confirmed: `src/pi_sdk/discover.rs` (all of it),
   `session_io.rs`, `protocol.rs`, `map_event.rs`, `map_event_tests.rs`,
   `protocol_tests.rs`, `discover_tests.rs` (after porting, see H4),
   `mock_pi.sh`, `BridgeWire::PiRpc` + its 2 arms
   (`bridge_sdk/session.rs:22,83,95-96`), re-exports at
   `pi_sdk/mod.rs:20,32`, mod lines for deleted files.
2. **KEEP `map_event_summary.rs`** — live dependency of `map_agent_event.rs`.
   Old envelope did not flag it as a trap; this is a correction.
3. Python scope is small: delete `ft_resolve_pi_bin` (`fast_task.py:96-110`),
   `PI_BIN_REMOTE` (:66), resolver test block (~:1666-1695), and the four
   negative assertions at :1105-1106, :1116-1117, :1383-1384.
4. Docs already crate-path — nothing to do beyond a final grep.
5. Kiss witness files needing edits (exact lists below):
   - `src/pi_sdk/kiss_coverage_tests.rs`: remove all discover refs (L3-8, L26),
     protocol refs (L46-47), JSON-mapper refs (L55), RPC session names +
     `BridgeWire::PiRpc` ref (L70-83, L103); keep NodeBridge ref (L104) if
     variant survives (it does — cursor uses it).
   - `src/coverage_kiss/test_kiss_static_coverage_05.rs`: prune
     `kiss_cov_pi_sdk_discover_auth_models` (L105-125: drop resolver names,
     keep auth/models names incl. `PI_MISSING_HINT` removal),
     `kiss_cov_pi_sdk_protocol` (L156-166: delete fn),
     `kiss_cov_pi_sdk_map_a` (L169-187: delete — typed mapper has its own
     probe tokens? NO: check `test_kiss_probe_static.rs` first; it already
     carries map_agent_event_end/last_assistant_text/aggregate_usage tokens),
     `kiss_cov_pi_sdk_spawn` (L190+: fine, no RPC names).
   - `src/pi_sdk/test_kiss_probe_static.rs`: drop `pi_write_abort`,
     `pi_send_new_session` from list b (L35-36); rest are live names.
   - Witness rule (prior exp_log): bare call-shaped tokens count;
     `stringify!` entries do NOT satisfy the scanner but are harmless to
     leave; qualified refs do not count for duplicate attribution.

## Execution order for next agent

1. Port two LIVE tests out of `discover_tests.rs` into a new
   `models_list_tests.rs` (or inline `#[cfg(test)]` in models_list.rs):
   `pi_list_models_timeout_env_clamps_and_defaults`,
   `list_pi_models_sync_reads_crate_registry` (+ its `write_exec_script`
   helper is only needed by resolver tests — do not port).
2. Delete Rust dead code per verified set above; run
   `CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib` then
   `cargo test --lib pi_sdk::` after each cluster.
3. Decide port-vs-drop for the three `parse_list_models_table` fixtures AND
   the `#[cfg(test)]` parser helpers in `models_list.rs`; if dropping,
   also remove `stringify!` refs (harmless but tidy) — same optional class
   for `providers_list.rs` `parse_list_providers_table` + tests.
4. Python sweep (scope in fact 3). Run `pytest tests -k fast_task` fast loop.
5. Kiss witness pruning exactly as listed; run `kiss check` alone before gates.
6. Full gates sequentially from `.malvin/gates` (use the gate runner or one
   line per shell invocation): ruff; kiss; clippy `--jobs 3 --all-targets
   --all-features -- -D warnings -W clippy::cargo`; pytest tests;
   `./admin/malvin_rust_test_gate.sh`.
7. Optional live smoke (acceptance 1–3): `malvin --model=pi:<prov>/<model>
   --do Hello` + `malvin models pi:` with no pi binary / no MALVIN_PI.

## Epistemic uncertainty

- H4 correction means blind whole-file delete of `discover_tests.rs` would
  silently lose live coverage while rust gate stays green (test count drops
  unnoticed) — hence port-first ordering is mandatory, not stylistic.
- Untested hypothesis (next agent should falsify cheaply): clippy `-D
  warnings` may flag unused `pub(crate)` items after deletions (e.g.
  anything left referencing session_io types); expect one fix-up pass.
- `kiss_cov_pi_sdk_map_a` deletion safety depends on whether kiss requires
  witness tokens for `map_agent_event_end.rs` symbols elsewhere — probe file
  (`test_kiss_probe_static.rs`) already lists them, so likely safe; verify
  with `kiss check` before full gates.

Predicted remaining effort: 45–70 min.
