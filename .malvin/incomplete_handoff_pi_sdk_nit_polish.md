# Incomplete handoff: pi::sdk residuals — nit 1 fixed, nit 2 scoped, gates pending

## What is done

1. Nit 1 (stale note) FIXED in `src/cli/models_cmd.rs` (print_pi_models):

   - Old: "Note: pi model list is cached; see `pi --help` to update it."
   - New: "Note: pi model list is built in-process; rows are shown only for
     authenticated providers."
   - Truthfulness evidence: crates.io
     `pi_agent_rust-0.1.23/src/models.rs::load_with_mode` rebuilds the list
     from built-ins on every `ModelRegistry::load_for_listing`; malvin never
     shells out (no resolver / `--list-models` code remains).
   - Repo-wide grep for "pi --help" now matches only this envelope, its
     exp_log.md status line, and the old audit envelope — no live code/docs.
   - No test asserts the old string (`models_cmd_tests.rs`,
     `models_cmd_auth_filter_tests.rs` checked).

2. Nit 2 (thread-leak assertion) SCOPED, not implemented:

   - Target behavior: explicit test asserting the named thread
     "malvin-pi-sdk" (spawned in `src/pi_sdk/runtime.rs::PiRuntime::start`)
     exists while a session lives and is gone after
     `end_coder_session()`/Drop (`PiRuntime::shutdown` joins the thread).
   - Entry point: mock-path client test in `src/pi_sdk/client_mock_tests.rs`
     (`pi_install_mock_env()` sets `MALVIN_TEST_NO_REAL_AGENT=1`;
     `test_env_lock()` guard required). Count threads before
     `begin_coder_session`, after a prompt, and after `end_coder_session`.
   - Counting options: read `/proc/self/task/*/comm` for `malvin-pi-sdk`
     (linux-only, consistent with existing unix-gated code); Rust std has no
     API to list foreign threads, so /proc is the practical route.

## What remains

- Implement the nit 2 test.
- Re-run all 5 gates sequentially (ruff / kiss / clippy / pytest /
  `./admin/malvin_rust_test_gate.sh`). The string edit has NOT been
  gate-verified this turn; the prior audit turn ran all gates green on an
  otherwise identical tree.
- Optionally delete this envelope once both nits are resolved and recorded.

## Next-agent starting position

- Working tree = prior audit-green tree + the single string edit above.
- Start at `src/pi_sdk/client_mock_tests.rs`; copy the env-guard pattern from
  `pi_sdk_client_mock_rpc_prompt_records_usage`; keep each test under 1.5 s
  (VISION.md) — the mock path spawns no real session, so the thread delta is
  the only new observation needed.
- After green gates: append a status line to exp_log.md; remove residual-nit
  language from future summaries.
