# Plan: Remove `malvin hello` and `do --repo-gates`

## User Request

- Remove `malvin hello`. Use `malvin do` instead.
- Remove `--repo-gates` as an option to `do`.

## Pre-removal state (historical)

The sections below describe the tree **before** this plan was executed. They are kept for context only; the live code no longer matches.

### `malvin hello` subcommand (removed)

Thin wrapper around `run_do` with a fixed prompt:

| File | Role |
|------|------|
| `src/cli/hello_flow.rs` | Defines `HelloArgs`, `HELLO_PROBE_PROMPT` (`"Hello"`), `run_hello()` → calls `run_do` with `repo_gates: false` |
| `src/cli/args.rs` | `Commands::Hello(HelloArgs)` with doc comment "Verify Cursor ACP connectivity" |
| `src/cli/entrypoint.rs` | Dispatches `Commands::Hello` → `run_hello` |
| `src/lib.rs`, `src/cli/mod.rs` | Module export and `pub use run_hello` |
| `default_prompts/docs/hello.md` | Embedded via `src/cli/command_docs.rs` |

Before delegating to `do`, `run_hello` calls `enable_probe_stdout_tee()` and `set_stdout_suppressed(false)`. These differ from plain `malvin do Hello` in edge cases:

| Concern | `malvin hello` | `malvin do Hello` |
|---------|----------------|-------------------|
| Interactive tee | yes (probe flag or TTY) | yes (TTY) |
| Piped stdout, no `MALVIN_FORCE_STDOUT_TEE` | probe tee forced on | may differ |
| `--background` | forces stdout unsuppressed | entrypoint suppresses stdout (`entrypoint.rs:108`) |

For typical interactive smoke checks the commands are equivalent. Deepswe already sets `MALVIN_FORCE_STDOUT_TEE=1` in `_relay_subprocess_stdout`, so the ops path is unaffected.

**Probe tee (hello-only):**

| File | Role |
|------|------|
| `src/output/stdout_tee_env.rs` | `PROBE_STDOUT_TEE` atomic; `enable_probe_stdout_tee()` consulted by `agent_stdout_tee_enabled()` |
| Only production caller | `src/cli/hello_flow.rs` |

**Hello referenced in match arms (must be cleaned):**

- `src/cli/config_defaults.rs` — loop/mini defaults
- `src/cli/entrypoint.rs` — `require_kiss_for_cli_command`
- `src/cli/entrypoint_checks.rs` — `ensure_default_malvin_config_file` for `Do | Hello`
- `src/cli/config_defaults_tests.rs` — parse/coverage test match arm
- `src/cli/bare_invoke_tests.rs` — `hello_subcommand_does_not_resolve_as_bare_kpop` (dedicated test; must be **deleted**)

**Docs:** `default_prompts/docs/malvin.md` commands table does **not** list `hello`. `hello.md` exists as a standalone `--doc` page.

**Ops layer (separate CLI, not malvin):**

| File | Role |
|------|------|
| `ops/deepswe_run.py` | `malvin_has_hello_subcommand()` probes `malvin hello --help`; `hello_probe_cmd()` prefers `malvin hello`, falls back to `malvin do Hello`; `run_malvin(command="hello")` and Click `hello` subcommand for Modal/host smoke tests |
| `ops/deepswe_modal.py` | `--command` choice includes `"hello"`; passes through to `deepswe_run` |
| `tests/test_deepswe_run_selftest_hello.py` | Python self-tests for deepswe `hello` (ops command name unchanged) |

### `do --repo-gates` (removed)

| File | Role |
|------|------|
| `src/cli/do_flow.rs` | `DoArgs.repo_gates: bool` (`#[arg(long, default_value_t = false)]`); `run_do_repo_gates_if_requested()` calls `repo_checks::run_repo_workspace_gates_no_kiss_clamp` when true |
| `default_prompts/docs/do.md` | Documents `--repo-gates` option and example |
| `default_prompts/docs/malvin.md` | Line 147: "`malvin do --repo-gates` and mid-loop gate iterations do **not** run discovery" |
| `src/cli/repo_checks/gate_run.rs` | Doc comment: "Used by `malvin do --repo-gates`" on `run_repo_workspace_gates_no_kiss_clamp` |

**Tests encoding current behavior:**

| File | Test |
|------|------|
| `src/cli/do_flow_tests.rs` | `cli_accepts_do_repo_gates` |
| `tests/do_stdout.rs` | `do_repo_gates_keeps_gate_diagnostics_off_stdout` |
| `tests/do_stdout_clamp.rs` | `do_repo_gates_does_not_invoke_kiss_clamp_without_kissconfig` |
| `src/cli_kiss_cov_smoke_tests.rs` | Asserts `repo_gates: false` on default `DoArgs` |
| `src/cli/do_flow_kiss_cov_tests.rs`, `src/cli_kiss_cov_smoke_tests_ext.rs` | References `run_do_repo_gates_if_requested` |

**Adjacent (unchanged):** Gate-loop commands (`code`, `tidy`, bare kpop, etc.) still run workspace gates via `repo_checks::run_repo_workspace_gates` in the kpop loop. `run_repo_workspace_gates_no_kiss_clamp` remains used by `src/cli/repo_checks/gate_run_tests.rs` — not dead code, only loses its CLI entry point.

## Requested Changes

1. Remove the `malvin hello` subcommand; connectivity probing becomes `malvin do Hello` (optionally `malvin do Hello --thoughts`).
2. Remove the `--repo-gates` flag from `malvin do`; `do` never runs workspace quality gates before the agent prompt.
3. Update docs, tests, and ops helpers that reference the removed surfaces.

## Behavioral impact

| Removed surface | Replacement | Footgun |
|-----------------|-------------|---------|
| `malvin hello` | `malvin do Hello` | After removal, **`malvin hello` does not error** — clap routes it like `malvin foobar`: bare kpop with request `"hello"` via `resolve_bare_command` (`src/cli/bare_invoke.rs`). Document in `do.md`; no code change planned to block this. |
| `malvin do --repo-gates` | none (use gate-loop commands) | Parse error after flag removal. |

## Q&A

### Q1. Keep the ops `deepswe_run hello` command, or rename it?

**Answer:** Keep it. It is a deepswe smoke-test entry point (Modal CIDR/auth, `--host` local probe), not a malvin subcommand. Only change the malvin argv it builds: always `[malvin_cmd, "do", "Hello", *malvin_args]`. Remove `malvin_has_hello_subcommand`, `_hello_subcommand_cache`, and fallback logic in `hello_probe_cmd`.

### Q2. Delete `run_repo_workspace_gates_no_kiss_clamp` and probe-tee machinery?

**Answer:** Remove probe-tee only (`enable_probe_stdout_tee`, `PROBE_STDOUT_TEE`, exports in `src/output/mod.rs`, kiss-cov refs). Keep `run_repo_workspace_gates_no_kiss_clamp` — still exercised by `gate_run_tests.rs`; update its doc comment to drop the `do --repo-gates` reference.

### Q3. Where should connectivity-probe guidance live after deleting `hello.md`?

**Answer:** Add a short note to `default_prompts/docs/do.md` (Examples or Related commands): `malvin do Hello` for a one-turn Cursor ACP smoke check. Note that bare `malvin hello` (no subcommand) runs kpop, not a connectivity probe. Delete `default_prompts/docs/hello.md`.

### Q4. Restore hello's `--background` / piped-tee behavior on `do Hello`?

**Answer:** No — out of scope. Callers that need forced tee (deepswe, CI) already set `MALVIN_FORCE_STDOUT_TEE=1`. Interactive `malvin do Hello` is sufficient for human smoke checks.

## Plan

### Phase 1 — Remove `malvin hello`

- [x] Delete `src/cli/hello_flow.rs`
- [x] Remove `hello_flow` module from `src/lib.rs`; remove `pub use run_hello` from `src/cli/mod.rs`
- [x] Remove `Commands::Hello` and `HelloArgs` import from `src/cli/args.rs`
- [x] Remove `Commands::Hello` dispatch from `src/cli/entrypoint.rs`; drop `run_hello` import
- [x] Remove `Commands::Hello` arms from `src/cli/config_defaults.rs`, `src/cli/entrypoint_checks.rs`, `src/cli/entrypoint.rs` (`require_kiss_for_cli_command`)
- [x] Remove `Commands::Hello` from `src/cli/command_docs.rs`; delete `default_prompts/docs/hello.md`
- [x] Add connectivity-probe example and bare-`hello` footgun note to `default_prompts/docs/do.md`
- [x] Remove probe-tee dead code from `src/output/stdout_tee_env.rs` and re-exports in `src/output/mod.rs`; update kiss-cov refs in `src/output/output_kiss_cov_tests.rs`
- [x] Delete test `hello_subcommand_does_not_resolve_as_bare_kpop` in `src/cli/bare_invoke_tests.rs`; remove `Commands::Hello` match arm in `src/cli/config_defaults_tests.rs`
- [x] Simplify `ops/deepswe_run.py`:
  - `hello_probe_cmd` → always `[malvin_cmd, "do", "Hello", *malvin_args]`
  - Remove `malvin_has_hello_subcommand`, `_hello_subcommand_cache`
  - Remove `_test_run_malvin_hello_uses_subcommand` and `_test_run_malvin_hello_falls_back_to_do_when_subcommand_missing`
  - Update `_test_hello_host_relays_stdout`: assert `"do"` and `"Hello"` in cmd (not `"hello"`)
  - Update docstrings that say "malvin hello" to "malvin do Hello" where referring to malvin argv
- [x] Leave deepswe Click command name `hello` and `ops/deepswe_modal.py` `--command hello` unchanged

**Validation:**

- `cargo test bare_invoke config_defaults command_docs do_flow`
- `cargo build`; `rg 'Commands::Hello|hello_flow|enable_probe_stdout_tee' src/` — no hits
- `rg 'malvin hello' src/` — no hits (footgun note lives in `default_prompts/docs/do.md`; ops docstrings updated separately)
- `malvin do Hello --help` parses; `malvin hello` resolves to bare kpop (same as `malvin foobar`), **not** a connectivity probe
- `python ops/deepswe_run.py self-test` — hello probe paths assert `[MALVIN_CMD, "do", "Hello", ...]`

### Phase 2 — Remove `do --repo-gates`

- [x] Remove `repo_gates` field from `DoArgs` in `src/cli/do_flow.rs`
- [x] Delete `run_do_repo_gates_if_requested` and its call in `prepare_do_run`; remove unused `repo_checks` import if no longer needed in this file
- [x] Remove `cli_accepts_do_repo_gates` and `repo_gates` assertions from `src/cli/do_flow_tests.rs`
- [x] Remove integration tests: `do_repo_gates_keeps_gate_diagnostics_off_stdout` (`tests/do_stdout.rs`), `do_repo_gates_does_not_invoke_kiss_clamp_without_kissconfig` (`tests/do_stdout_clamp.rs`); keep remaining clamp tests that assert default `do` behavior
- [x] Update `src/cli_kiss_cov_smoke_tests.rs` and `src/cli/do_flow_kiss_cov_tests.rs` / `src/cli_kiss_cov_smoke_tests_ext.rs` to drop `repo_gates` / `run_do_repo_gates_if_requested` refs
- [x] Update `src/cli/entrypoint_checks.rs` test struct literal (`repo_gates: false` → field gone)
- [x] Update docs: remove `--repo-gates` section and example from `default_prompts/docs/do.md`; rewrite line 147 in `default_prompts/docs/malvin.md` (mid-loop gate iterations only — drop `do --repo-gates` clause)
- [x] Update doc comment on `run_repo_workspace_gates_no_kiss_clamp` in `src/cli/repo_checks/gate_run.rs`

**Validation:**

- `cargo test do_flow do_stdout do_stdout_clamp` — passes; no tests reference `--repo-gates`
- `malvin do --help` — no `--repo-gates` flag
- `malvin do --repo-gates "x"` — clap parse error
- `rg '--repo-gates|run_do_repo_gates|DoArgs.*repo_gates' src/ default_prompts/docs/` — no hits (`src/repo_gates/` module name is expected elsewhere)
- `cargo test gate_run` — `run_repo_workspace_gates_no_kiss_clamp` tests still pass
