# Malvin repo — tooling, layout, quality gates

## Required checks (repo root)

- `ruff check .`
- `kiss check .` (**not** bare `kiss`). See `.kissignore`.
- `pytest -sv tests` (minimal Python smoke; primary tests are Rust). If a test imports the repo as a package, run from repo root with `PYTHONPATH=.`.
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings` (`clippy::doc_markdown`: wrap code-like tokens in `//!`/`///` in backticks—bare identifiers fail under `-D warnings`)

Pre-commit **`cargo-clippy`** (must match `.pre-commit-config.yaml` `entry:` verbatim):

```text
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc
```

Pre-commit also runs `ruff check .`, `kiss check .`, and `admin/check_untracked.sh` (fails if untracked `.rs`/`.py` sources exist). It does **not** run `cargo test` or `pytest`; run the full suite before merge.

If the workspace **ripgrep/search tool** errors (e.g. I/O), run **`rg` from a repo-root shell** instead. **KPOP experiment logs**, `malvin models` parser/ANSI caveats, and the same search fallback are indexed in **`.llm_style/malvin_debugging.md`**.

### Untracked source files (`admin/check_untracked.sh`)

Fails when `*.rs` or `*.py` exist under the repo but are not tracked (`git ls-files --others --exclude-standard`). **Agents** that must not run `git` cannot `git add`; fold new tests into an existing tracked `tests/*.rs` (e.g. `cli_parity.rs`) or ask the user to stage. Keeps pre-commit green without bypassing the hook.

## Hard constraints

- **Never edit** `.kissconfig`.
- **Do not run git** in automated assistance for this project; users stage/commit locally.
- **Rust** `edition = "2024"`, `rust-version = "1.87"` in `Cargo.toml`.
- **`.kissignore`** may exclude paths; still run `kiss check .` on the analyzed set.

## Crate layout (high level)

| Area | Location |
|------|----------|
| Library entry | `src/lib.rs` (re-exports + deprecated `malvin::agent` shim) |
| Binary entry | `src/main.rs` → `src/cli/` |
| ACP JSON-RPC / session / agent client | `src/acp/` — **many** pieces are `include!`d (see below) |

**Binary vs library:** `src/cli/` is part of the **`malvin` binary crate**, not `malvin` the library. `pub(crate)` fields on `AgentClient` (e.g. `timing`) are visible inside `src/lib.rs` code only—**not** from `src/cli/`. Use public methods on the library client (e.g. `attach_run_timing_for_session`) or keep field access in lib modules (`src/orchestrator/`, …).
| Invocation argv | `src/invocation.rs` |
| Log path display | `src/log_paths.rs` |
| Run artifacts | `src/artifacts.rs` |
| Orchestrator | `src/orchestrator/`, `src/review_sync.rs`; `#[cfg(test)]` `src/orchestrator_tests.rs` |
| Post-run metrics hint | `src/post_run_hint/report.rs` — post-run stderr line; called from `src/orchestrator/` and KPOP after ACP bodies |
| Run timing | `src/run_timing/mod.rs` + `src/run_timing/report.rs` — `malvin code` and `malvin kpop`: `run_timing.json` + one stdout summary after workflow, before the post-run metrics hint; LLM vs retry/backoff; see root `grounding.md` |
| Prompts | `src/prompts/` + `default_prompts/` |

### ACP `include!` assembly (kiss dependency depth)

Navigate by **include file names** (not only `mod` tree): e.g. `tee_strip_body.inc`, `ops_body.inc` (`maybe_tee_log`, reviewer pair), `reader_inline.inc`, `agent_bundle.inc`, `transport/*.rs`, `coalesce.rs`.

**Included `.rs` files** (e.g. `transport/command.rs` pulled into `acp/mod.rs`) **inherit the parent module’s `use`**—types like `Path` are not imported locally unless the include parent brings them.

## Child health + ACP silence (`src/child_health/`)

TRIGGER: child health module layout  
ADVICE: Library module at `src/child_health/mod.rs` with `linux.rs`, `macos.rs` (`libproc` + `errno`/`libc`), `other.rs`, and `tests.rs` when `kiss` `lines_per_file` requires. Wired from `src/lib.rs` (`mod child_health`); RPC wait in `src/acp/transport/rpc.rs` (`child_pid` on `AcpStdioRpc`). `src/coverage_kiss.rs` `stringify!` for public helpers.

TRIGGER: process_absent cannot_sample  
ADVICE: **`process_absent`**: OS says PID row missing (`/proc` `NotFound`; macOS `proc_pidinfo` + `errno == ESRCH`). **`cannot_sample`**: I/O/parse failure or ambiguous read—`exists: true`, `counters_trusted: false`, zero placeholders. Do not conflate with “gone” (user-facing `acp child process is not running`).

TRIGGER: counters_trusted progress  
ADVICE: **`health_indicates_progress`** compares CPU/context/thread fields only when **both** snapshots have `counters_trusted`. If the first sample is untrusted, do **not** infer progress from a lone trusted second read (typical Linux `/proc` counters would always look “busy”). `silence_grace_for_rpc_timeout` clamps `rpc_timeout/8` to 50–250ms.

TRIGGER: rpc_wait_response health race  
ADVICE: After the silence `sleep(rpc_timeout)`, use `tokio::select!` so the JSON-RPC **`oneshot`** and **`evaluate_after_acp_silence`** (grace sleep inside) are polled together—inbound responses during grace must return success, not `AppearsHung`. Regression: `transport_tests::rpc_response_arriving_during_child_health_grace_is_delivered`.

TRIGGER: voluntary_ctxt switches parse  
ADVICE: In `child_health/linux.rs` **`parse_status_voluntary_ctxt`**, after `strip_prefix("voluntary_ctxt_switches:")`, use **`rest.trim().parse::<u64>()`**—`trim_start()` alone leaves a trailing **`\r`** on the value token and **`u64` parse returns `Err`**, so voluntary context switches are dropped and progress detection weakens. Regression: `child_health::tests::linux_parse::voluntary_ctxt_parses_when_value_has_trailing_cr`.

## Post-run metrics hint

- **Code:** `src/post_run_hint/report.rs` — `finish_and_write_report` / `finish_post_run_hint_then_return`; prints a stable **“not measured”** stderr line only. **`src/post_run_hint/mod.rs`** documents that **gross/net metering and git tree snapshots were removed**—there is no yield/gross/net computation in product code.
- **Streams / ordering:** Stable **“not measured”** line → **`eprintln!` (stderr)** only — see root `grounding.md`. `finish_post_run_hint_then_return` runs after the workflow/KPOP ACP body, before CLI `DONE` / `end_coder_session` (or equivalent).
- **Contract tests:** `tests/cli_parity.rs` asserts the message does **not** contain `"git"` (legacy git-tree metering must not leak into user copy).

## Run timing (`malvin code` / `malvin kpop`)

- **Code:** `src/run_timing/` (`mod.rs`, `report.rs`); orchestrator sets `AgentClient::timing` and finalizes after `run_with_coder_session`; KPOP attaches timing and calls the same finalizer; ACP `client_impl.inc` / `ops_body.inc` record `session/prompt` duration and bounded-retry sleeps.
- **Artifacts:** `run_timing.json` in the run directory; stdout summary line (see root `grounding.md`) is emitted **before** the post-run metrics hint on the main code path.
- **Dual failure:** If timing I/O and workflow/ACP both fail, return the **primary** error first (`prefer_primary_errors_over_timing` in `src/orchestrator/mod.rs`, `merge_acp_and_timing_after_post_hint` in `src/cli/kpop_flow.rs`).
- **Rustdoc:** Helpers that run *after* timing + hint I/O must not read as reordering streams vs `grounding.md`—they may only merge `Result`s once stdout/stderr order is already established in the caller.

## ACP traces, coalescing, tee

- **Trace format:** After `AcpSession::prompt`, trace may start with plaintext `Command: …\n` (from `invocation`), then JSON lines from agent stdout—not guaranteed pure JSONL when that prelude exists.
- **Tee:** `maybe_tee_log` (in `ops_body.inc`) reads the **whole** trace file; `strip_trace_invocation_line_for_tee` (`tee_strip_body.inc`) drops the duplicate prelude line. No-newline `Command:`-only buffers strip to empty (documented + tested).
- **Coalescing:** Verbose/trace paths track **Unicode scalar counts** per buffer in `coalesce.rs` to avoid repeated full-buffer `chars().count()` in flush loops.

## Tests

- **Node:** Many ACP tests use executable Node scripts as mock `agent acp` children; `node` must be on `PATH` or handshake tests fail. Spawns that need a minimal UNIX layout use **`prepend_standard_path_for_child`** (`src/acp/transport/command.rs`) so `#!/usr/bin/env node` resolves.
- **Brittle source tests:** Prefer behavioral tests over `include_str!` substring checks on `mod.rs` that break on refactors.
- **CLI / gitignore guards:** Cross-cutting behavioral checks and `git check-ignore` fixtures often live in `tests/cli_parity.rs` (alongside ACP spawn string guards).
- **Grounding vs code:** `tests/cli_parity.rs` may `include_str!` root `grounding.md` and implementation files (e.g. `src/post_run_hint/report.rs`) so documented stdout/stderr post-run behavior stays aligned with sources—extend when stream contracts change.

## Review sync, `review.md`, shared output

TRIGGER: RunArtifacts review paths  
ADVICE: Use `RunArtifacts::artifact_review_md()` / `workspace_review_md()` in `src/artifacts.rs` for workspace ↔ run artifact `review.md` paths; avoid duplicating `run_dir.join("review.md")` / `work_dir.join("review.md")` at call sites.

TRIGGER: sync_review_then_is_lgtm  
ADVICE: `src/review_sync.rs` — `sync_review_then_is_lgtm` returns **`io::Result<bool>`** (propagate read/write with `?`); map to `AgentError` / `WorkflowError` in `ops_body.inc` and `orchestrator/review_loop.rs`. Do not treat sync I/O failure as “not LGTM.”

TRIGGER: reviewer pair order regression  
ADVICE: `tests/cli_parity.rs` **`reviewer_pair_ops_preserves_review_sync_lgtm_before_kpop_order`** `include_str!`s `src/acp/ops_body.inc` and asserts source order: review `session/prompt` → `sync_review_then_is_lgtm(...)` → kpop `session/prompt`. Pair with behavioral tests in `src/review_sync.rs` (not only substring guards).

TRIGGER: shared stdout stderr output  
ADVICE: `src/output.rs` — line-oriented helpers (`format_line`, `print_stdout_line`, …). Align `#[must_use]` with sibling APIs if plain `cargo clippy` warns; pre-commit allows `-A clippy::must_use_candidate`.

TRIGGER: redundant_pub_crate  
ADVICE: `clippy::redundant_pub_crate`: in a non-`pub` submodule, prefer `pub struct` over `pub(crate) struct` when the type is only re-exported at the parent (e.g. `acp/session_types.rs` → `acp/mod.rs`).

### Repo-wide string contracts (renames, banned fragments)

When removing or renaming a user-facing term, **`rg` the whole repository** (implementation, `grounding.md`, `default_prompts/`, `.cursorrules`, `.llm_style/`, `_kpop/` logs). A short **forbidden substring** may appear inside unrelated English words—verify with context, not only exact tokens. In **learn/review prompts**, distinguish **agent pacing** (latency, thoroughness) from **post-run metrics** language in code (`post_run_hint`). **`tests/cli_parity.rs`** asserts `grounding.md` matches stderr contracts when implementation uses `eprintln!` for the post-run hint; if docs lag, tests fail before runtime.

## kiss

Enforces lines-per-file, call counts, duplication, etc. Use `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` so symbols stay visible. Split modules when limits hit (e.g. extract `report.rs`, thin orchestrator `run()` helpers when `calls_per_function` fires). Run `kiss check .` during multi-step work—not only at the end.

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
- **Retriable strings:** `agent_string_is_retriable` uses ASCII-lowercased `contains` checks—add **narrow** substrings for transient transport/session teardown (e.g. writable/readable iterable closed), not broad patterns that mask logic errors.
- **`timeout` wording:** A bare `contains("timeout")` can match **config/validation** copy (`timeout_ms`, `grpc_timeout_ms`). The policy uses `timeout_word_without_identifier_false_positive` in `retry_policy.inc` (skip when `timeout_` appears) while keeping phrases like `timed out`. Regress in `agent_bundle.inc` `retry_policy_tests`.
- **Sleep/break loop:** `backoff_after_agent_failure` in `src/acp/client_impl.inc` (included via `agent_bundle.inc`).
- **Included in:** `agent_bundle.inc` pulls `retry_policy.inc`, `ops_body.inc`, `client_impl.inc`.
- **Unit tests:** `retry_policy_tests` in `src/acp/agent_bundle.inc` (policy helpers are not only tested from integration tests).
- **User-facing exhaustion messages:** `client_impl.inc` formats `failed after {retries} retries` using `retries = attempts_used.saturating_sub(1)` (post-first-failure attempts), not raw `MAX_AGENT_ATTEMPTS`.

TRIGGER: ACP upgrade plan eprintln  
ADVICE: Upgrade-plan `Err`: **single** `eprintln!` at `src/cli/mod.rs` (not duplicated in `client_impl.inc`); see `retry_policy.inc` / `client_impl.inc`.

- **Ad-hoc task specs:** `_malvin/**/plan.md` may hold one-off agent instructions—implement when the user cites that path; product/bootstrap `plan.md` remains the shipped template story (`init_cmd`).

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

- **MSRV / edition:** `edition = "2024"`, `rust-version = "1.87"` in `Cargo.toml`; mention in `README.md` if documenting toolchain.
- **Orchestrator prompt stems:** `prompt_md_stem` / `strip_suffix(".md")` in `src/orchestrator/` — avoid `len()-3` slicing.
- **Prompts `include_str!`:** defaults live under `default_prompts/`; paths in `src/prompts/mod.rs`.
