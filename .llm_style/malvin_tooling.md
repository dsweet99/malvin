# Malvin repo — tooling, layout, quality gates

## Required checks (repo root)

- `ruff check .`
- `kiss check .` (**not** bare `kiss`). See `.kissignore`.
- `pytest -sv tests` (minimal Python smoke; primary tests are Rust)
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`

Pre-commit **`cargo-clippy`** (must match `.pre-commit-config.yaml` `entry:` verbatim):

```text
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc
```

Pre-commit also runs `ruff check .`, `kiss check .`, and `admin/check_untracked.sh` (fails if untracked `.rs`/`.py` sources exist). It does **not** run `cargo test` or `pytest`; run the full suite before merge.

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
| Prompts | `src/prompts/` + `default_prompts/` |

### ACP `include!` assembly (kiss dependency depth)

Navigate by **include file names** (not only `mod` tree): e.g. `tee_strip_body.inc`, `ops_body.inc` (`maybe_tee_log`, reviewer pair), `reader_inline.inc`, `agent_bundle.inc`, `transport/*.rs`, `coalesce.rs`.

## ACP traces, coalescing, tee

- **Trace format:** After `AcpSession::prompt`, trace may start with plaintext `Command: …\n` (from `invocation`), then JSON lines from agent stdout—not guaranteed pure JSONL when that prelude exists.
- **Tee:** `maybe_tee_log` (in `ops_body.inc`) reads the **whole** trace file; `strip_trace_invocation_line_for_tee` (`tee_strip_body.inc`) drops the duplicate prelude line. No-newline `Command:`-only buffers strip to empty (documented + tested).
- **Coalescing:** Verbose/trace paths track **Unicode scalar counts** per buffer in `coalesce.rs` to avoid repeated full-buffer `chars().count()` in flush loops.

## Tests

- **Node:** Many ACP tests use executable Node scripts as mock `agent acp` children; `node` must be on `PATH` or handshake tests fail.
- **Brittle source tests:** Prefer behavioral tests over `include_str!` substring checks on `mod.rs` that break on refactors.

## kiss

Enforces lines-per-file, call counts, duplication, etc. Use `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` so symbols stay visible. Split modules when limits hit.

## Breaking API notes

- Document consumer-visible removals (e.g. old `malvin::agent` paths) in **`CHANGELOG.md`**.

## CLI

- `src/cli/args.rs`, `mod.rs`, `shared_opts.rs`; `tee_startup_stdout` gates startup `Command:` + plan echo vs `--no-tee`.

## Reviewer workflow (conceptual)

After **review** prompt: sync workspace review to artifact, **`is_lgtm`** on artifact before **kpop** prompt—implementation in `src/acp/ops_body.inc` / orchestrator, not a single legacy `src/agent/ops.rs` file.
