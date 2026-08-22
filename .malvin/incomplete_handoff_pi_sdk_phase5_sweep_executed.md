# Incomplete handoff: Phase 5 sweep EXECUTED — gates kiss/clippy/pytest/rust unverified

Status: deletion sweep applied and compile-verified. ruff PASS.
`cargo check --all-targets` PASS. `cargo test --lib pi_sdk::` = 28/28 PASS.
**Not yet run:** kiss, clippy, full pytest, rust test gate.

Work dir `/home/dsweet/Projects/malvin`. Exp log:
`_run/exp_log_20260822_040428_1z8fhizh_g1.md` (H1–H5).

## Done

1. Ported live tests from deleted `discover_tests.rs` into
   `src/pi_sdk/models_list_tests.rs`, wired via `#[cfg(test)] #[path]`
   child module of `models_list` (import is `use super::{...}`, not
   `super::models_list::...` — the child's super IS models_list).
2. Deleted: `src/pi_sdk/{discover.rs, discover_tests.rs, session_io.rs,
   protocol.rs, protocol_tests.rs, map_event.rs, map_event_tests.rs,
   mock_pi.sh}`. Kept `map_event_summary.rs` (live).
3. `src/pi_sdk/mod.rs`: dropped discover/session_io/protocol/map_event mods
   + re-exports; added `models_list_tests`.
4. `bridge_sdk/session.rs`: removed `BridgeWire::PiRpc` variant + both arms;
   `bridge_sdk/mod.rs`: removed `stringify!(PiRpc)` witness.
5. Kiss witnesses pruned: `pi_sdk/kiss_coverage_tests.rs` (rewritten),
   `coverage_kiss/test_kiss_static_coverage_05.rs` (removed resolver names,
   PI_MISSING_HINT, protocol block, map_a block, models_list_helpers block,
   kiss_cov_pi_discover), `pi_sdk/test_kiss_probe_static.rs` (removed
   pi_write_abort/pi_send_new_session).
6. Python (`src/python/fast_task.py`): removed `ft_resolve_pi_bin`,
   `PI_BIN_REMOTE`, MALVIN_PI resolver test block, all negative
   MALVIN_PI/mount assertions (ruff clean after dropping now-unused
   `mounts` binding).

## Remains (execute in this order)

1. **`kiss check` FIRST** — biggest uncertainty: my rewritten
   `kiss_coverage_tests.rs` dropped stringify! witnesses for the
   `#[cfg(test)]` parser helpers in `models_list.rs` /
   `providers_list.rs` (`parse_list_models_table`, `is_separator_line`,
   header/columns helpers, etc.). If kiss reports those files sub-90%:
   either re-add `stringify!(name)` lines or delete the dead
   `#[cfg(test)]` parsers entirely (preferred; plan calls them optional
   deletions). Also possible cascade in coverage_kiss contract files.
2. clippy: `cargo clippy --jobs 3 --all-targets --all-features -- -D
   warnings -W clippy::cargo` (expect maybe unused-import nits in edited
   files).
3. `pytest tests` (fast_task self-test must pass post-sweep).
4. `./admin/malvin_rust_test_gate.sh` (full rust suite).
5. Update `exp_log.md` one-liner + flip handoff to complete.

## Next-agent start

Run `kiss check` in `/home/dsweet/Projects/malvin`; fix findings per step 1;
then steps 2–4 sequentially, one gate line at a time (memory rules).
Predicted: 20–40 min.
