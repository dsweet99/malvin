# Incomplete handoff: pi::sdk in-process — KPop audit: gates green; plan satisfied; two residual nits

Status: **no code edits this turn** (audit turn). All 5 requested gates were
re-run fresh and PASS:

- `ruff check` — All checks passed
- `kiss check` — NO VIOLATIONS
- `cargo clippy --jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo` — Finished clean
- `pytest tests` — 157 passed
- `./admin/malvin_rust_test_gate.sh` — 843 tests, all passed

## Plan-vs-tree audit result

- Phase 0 ✓: `rust-version = "1.96"` + repo-root `rust-toolchain.toml`
  (`channel = "1.96.0"`); crates.io dep
  `pi = { package = "pi_agent_rust", version = "0.1.23", default-features = false, features = ["sqlite-sessions"] }`;
  compile-only probe of `pi::sdk::SessionOptions` in `src/pi_sdk/mod.rs`.
- Phase 1 ✓: typed mapper `map_agent_event.rs` (+ `_end`, `_summary`),
  ported tests in `map_agent_event_tests.rs`.
- Phase 2 ✓: `runtime.rs` (dedicated asupersync thread, mpsc/oneshot,
  AbortHandle), `session.rs` (`PiEmbeddedSession`, biased drain with
  reply-cache invariant), `SdkSession::Pi` enum arm.
- Phase 3 ✓ (chosen policy option 1+2): `isolated_bash.rs` wraps the builtin
  bash via `SessionOptions::tool_factory` → `malvin_std_command`
  process-group isolation + `note_session_affiliated_pid`; embedded mem-watch
  (`watch_embedded_memory`) reuses the same limit file over malvin's own
  baseline; `--no-force` fails before session creation
  (`client_mock_tests::pi_sdk_noforce_fails_fast`).
- Phase 4 ✓: `models_list.rs` / `providers_list.rs` / `auth.rs` use
  `AuthStorage` + `ModelRegistry::load_for_listing`; no `resolve_pi_bin`,
  no `MALVIN_PI`, no RPC protocol files left in `src/pi_sdk/`
  (discover/session_io/protocol/map_event/mock_pi.sh deleted).
- Phase 5 ✓ mostly: docs (README, models.md, malvin.md "Pi SDK" bullet) and
  `ops/fast_task.py` describe the crate path; fake-session client tests cover
  usage / last-response / empty-result / early-AgentEnd.

Acceptance items: 2,4,5(partial),7,8,9 verified directly in tree this turn;
1 and 3 evidenced by g1-retry2 live smoke recorded in exp_log.md
(`models pi:` + `--do Hello` with pi binary unreachable).

## Residual nits (non-blocking, next agent may polish)

1. `src/cli/models_cmd.rs` still prints
   `"Note: pi model list is cached; see `pi --help` to update it."` — the
   `pi --help` mention is stale now that listing comes from the crate
   registry (plan §Phase 4 keeps line format, not this wording).
2. Acceptance 5 ("shutdown does not leave extra threads or leaked asupersync
   runtimes") is covered only indirectly (Drop joins the runtime thread);
   there is no explicit begin/end pair test asserting a thread-count delta.

## Next-agent start

If polishing: fix nit 1 (one-line string change + grep for other stale
`pi --help` strings), optionally add a thread-delta test around
`ensure_coder_session`/`end_coder_session`. Re-run the 5 gates after.
