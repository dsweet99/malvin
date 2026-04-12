# Malvin repo — tooling, layout, quality gates

## Required checks (repo root)

- `ruff check .`
- `kiss check .` (**not** bare `kiss`). See `.kissignore`.
- `pytest -sv tests` (minimal Python smoke; primary tests are Rust). If a test imports the repo as a package, run from repo root with `PYTHONPATH=.`.
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

Pre-commit **`cargo-clippy`** (must match `.pre-commit-config.yaml` `entry:` verbatim):

```text
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc
```

Pre-commit also runs `ruff check .`, `kiss check .`, and `admin/check_untracked.sh` (fails if untracked `.rs`/`.py` sources exist). It does **not** run `cargo test` or `pytest`; run the full suite before merge.

### Untracked source files (`admin/check_untracked.sh`)

Fails when `*.rs` or `*.py` exist under the repo but are not tracked (`git ls-files --others --exclude-standard`). **Agents** that must not run `git` cannot `git add`; fold new tests into an existing tracked `tests/*.rs` (e.g. `cli_parity.rs`) or ask the user to stage. Keeps pre-commit green without bypassing the hook.

## Hard constraints

- **Never edit** `.kissconfig`.
- **Do not run git** in automated assistance for this project; users stage/commit locally.
- **Rust** `edition = "2024"`, `rust-version = "1.85"` in `Cargo.toml`.
- **`.kissignore`** may exclude paths; still run `kiss check .` on the analyzed set.

## Crate layout (high level)

| Area | Location |
|------|----------|
| Library entry | `src/lib.rs` (re-exports + deprecated `malvin::agent` shim) |
| Binary entry | `src/main.rs` → `src/cli/` |
| ACP JSON-RPC / session / agent client | `src/acp/` — **many** pieces are `include!`d (see below) |
| Invocation argv | `src/invocation.rs` |
| Log path display | `src/log_paths.rs` |
| Run artifacts | `src/artifacts.rs` |
| Orchestrator | `src/orchestrator/`, `src/review_sync.rs`; `#[cfg(test)]` `src/orchestrator_tests.rs` |
| Edit efficiency | `src/edit_efficiency/` — meter, reports; wired from `src/orchestrator/` (prompt-boundary checkpoints) |
| Prompts | `src/prompts/` + `default_prompts/` |

### ACP `include!` assembly (kiss dependency depth)

Navigate by **include file names** (not only `mod` tree): e.g. `tee_strip_body.inc`, `ops_body.inc` (`maybe_tee_log`, reviewer pair), `reader_inline.inc`, `agent_bundle.inc`, `transport/*.rs`, `coalesce.rs`.

**Included `.rs` files** (e.g. `transport/command.rs` pulled into `acp/mod.rs`) **inherit the parent module’s `use`**—types like `Path` are not imported locally unless the include parent brings them.

## Edit efficiency

- **Code:** `src/edit_efficiency/` — `EditEfficiencyMeter`, tree/byte diffs (`similar`, Myers-style opcodes), git helpers, `finish_and_write_report`, tests in `meter_tests.rs`.
- **Streams / ordering:** Successful one-line summary → **`println!` (stdout)**; not measured or `finish` failure → **`eprintln!` (stderr)** — see root `grounding.md`. `finish_edit_efficiency_then_return` runs after the workflow/KPOP ACP body, before CLI `DONE` / `end_coder_session` (or equivalent).
- **Orchestration:** Checkpoints run at **prompt boundaries** (coder / reviewer–kpop), not necessarily every in-session tool invocation; see `src/orchestrator/mod.rs` and `review_loop.rs`.
- **Fields:** `checkpoint_calls` = successful `checkpoint()` only; `gross_diff_steps` = those plus an extra step when `finish()` applies a trailing tree diff after the last checkpoint.
- **Byte totals vs `CPython`:** Doc in `byte_cost.rs` — not bit-identical to `difflib.SequenceMatcher`; algorithms differ.
- **Git temp index:** Prefer a temp **directory** and a nonexistent index path until git creates it; an empty index file can break git operations.

## ACP traces, coalescing, tee

- **Trace format:** After `AcpSession::prompt`, trace may start with plaintext `Command: …\n` (from `invocation`), then JSON lines from agent stdout—not guaranteed pure JSONL when that prelude exists.
- **Tee:** `maybe_tee_log` (in `ops_body.inc`) reads the **whole** trace file; `strip_trace_invocation_line_for_tee` (`tee_strip_body.inc`) drops the duplicate prelude line. No-newline `Command:`-only buffers strip to empty (documented + tested).
- **Coalescing:** Verbose/trace paths track **Unicode scalar counts** per buffer in `coalesce.rs` to avoid repeated full-buffer `chars().count()` in flush loops.

## Tests

- **Node:** Many ACP tests use executable Node scripts as mock `agent acp` children; `node` must be on `PATH` or handshake tests fail. Spawns that need a minimal UNIX layout use **`prepend_standard_path_for_child`** (`src/acp/transport/command.rs`) so `#!/usr/bin/env node` resolves.
- **Brittle source tests:** Prefer behavioral tests over `include_str!` substring checks on `mod.rs` that break on refactors.
- **CLI / gitignore guards:** Cross-cutting behavioral checks and `git check-ignore` fixtures often live in `tests/cli_parity.rs` (alongside ACP spawn string guards).

## kiss

Enforces lines-per-file, call counts, duplication, etc. Use `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` so symbols stay visible. Split modules when limits hit.

## Breaking API notes

- Document consumer-visible removals (e.g. old `malvin::agent` paths) in **`CHANGELOG.md`**.

## CLI

- `src/cli/args.rs`, `mod.rs`, `shared_opts.rs`; `tee_startup_stdout` gates startup `Command:` + plan echo vs `--no-tee`.
- **Default model:** `DEFAULT_CLI_MODEL` in `shared_opts.rs`; `malvin models` footer must use the same constant (see `tests/cli_parity.rs`).

## `malvin init`, `plan.md`, env

- **Implementation:** `src/cli/init_cmd.rs` — templates from `default_repo/`, `admin/check_untracked.sh`, then tooling bootstrap (order documented in `plan.md`).
- **Tests:** `tests/init_pre_commit.rs` spawns the real binary with a minimal `PATH` to assert fail-fast when `pre-commit` is missing (avoids relying on `malvin::test_utils` from the binary crate).
- **Agent on PATH:** `src/env_path.rs` — `lookup_bin_on_path`, `agent_or_cursor_agent_bin()` (same `agent` → `cursor-agent` preference as `resolve_agent_bin` in `ops_body.inc`).

## ACP bounded retry (where it lives)

- **Policy:** `src/acp/retry_policy.inc` (`MAX_AGENT_ATTEMPTS`, `plan_agent_retry`, retriable / upgrade-plan strings).
- **Sleep/break loop:** `backoff_after_agent_failure` in `src/acp/client_impl.inc` (included via `agent_bundle.inc`).
- **Included in:** `agent_bundle.inc` pulls `retry_policy.inc`, `ops_body.inc`, `client_impl.inc`.

## Reviewer workflow (conceptual)

After **review** prompt: sync workspace review to artifact, **`is_lgtm`** on artifact before **kpop** prompt—implementation in `src/acp/ops_body.inc` / orchestrator, not a single legacy `src/agent/ops.rs` file.

Root **`review.md`** is the working reviewer checklist (“problems only” / resolved). After fixing issues, update it so it does not stay stale versus `grounding.md` and the code.

## KPOP `--p-creative` / MBC2

- **Selection:** `src/kpop_acp_prompt.rs` — `kpop_acp_user_prompt`, `KpopAcpPromptPick`, `CREATIVE_MIN_INTERACTION`, `kpop_standalone_outbound_prompt_count`, `KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE`.
- **Session:** `src/acp/ops_body.inc` `run_kpop_flow_once` — when `p_creative > 0`, sends continuation `session/prompt` rounds (after main + optional `learn.md`) so outbound `interaction_index` can reach **3** before the MBC2 branch may apply; sibling traces like `*_creative_pad*.log`, `*_creative_roll.log` next to the primary `kpop` trace.
- **Prompts:** `default_prompts/mbc2.md`; embedding / defaults in `src/prompts/mod.rs`; CLI in `src/cli/args.rs` (`KpopArgs`).

## Rust edition 2024 + clippy (malvin)

- **`gen` is a keyword:** do not call `rng.gen()`; use e.g. `rand::distributions::{Distribution, Uniform}` and `sample(rng)`.
- **Float guards:** prefer `!x.is_finite() || x <= 0.0` over `!(x > 0.0)` where clippy flags `neg_cmp_op_on_partial_ord` (NaN / ordering).
- **`use` placement:** avoid `use` items after other statements in a block (`clippy::items_after_statements`); lift imports to the enclosing module (e.g. `rand` in `agent_bundle.inc` for `ops_body.inc`).
- **Async + RNG:** `thread_rng()` / `ThreadRng` is not `Send`; do not hold it across `.await`. For multiple `session/prompt` rounds in one async fn, use one `rand::rngs::StdRng::from_entropy()` (or seed) and `&mut rng`.
- **kiss arity:** if `arguments_per_function` fires, group parameters in a struct (same pattern as `KpopAcpPromptPick`).

## Keyword index (moved from `style.md` surface)

- **MSRV / edition:** `edition = "2024"`, `rust-version = "1.85"` in `Cargo.toml`; mention in `README.md` if documenting toolchain.
- **Orchestrator prompt stems:** `prompt_md_stem` / `strip_suffix(".md")` in `src/orchestrator/` — avoid `len()-3` slicing.
- **Prompts `include_str!`:** defaults live under `default_prompts/`; paths in `src/prompts/mod.rs`.
