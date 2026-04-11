# Malvin repo — tooling, layout, quality gates

## Required checks (repo root)

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `ruff check .`
- `kiss check .` (**not** bare `kiss`)
- `pytest -sv tests` (minimal Python smoke; primary tests are Rust)

Pre-commit (`.pre-commit-config.yaml`) runs **ruff**, **cargo clippy** (with extra `-W`/`-A` flags), and **kiss** only—**not** `cargo test` or `pytest`; run the full suite before merge.

## Hard constraints

- **Never edit** `.kissconfig`.
- **Do not run git** in automated assistance for this project; users stage/commit locally.
- **Rust** `edition = "2024"`, `rust-version = "1.85"` in `Cargo.toml`.
- **`.kissignore`** may exclude paths; still run `kiss check .` on the analyzed set.

## Crate layout (high level)

| Area | Location |
|------|----------|
| Library entry | `src/lib.rs` |
| Binary entry | `src/main.rs` → `src/cli/` |
| ACP JSON-RPC | `src/acp/` (`reader`, `transport`, `mod`) |
| Agent + tee | `src/agent/` (`client`, `ops`, `pair`, `tee_strip`) |
| Invocation argv | `src/invocation.rs` (space-joined display; not shell-safe) |
| Log path display | `src/log_paths.rs` (`format_logs_dir`) |
| Run artifacts | `src/artifacts.rs` |
| Orchestrator | `src/orchestrator/`, `src/review_sync.rs` |
| Prompts | `src/prompts/` + `default_prompts/` |

## ACP traces and tee

- **Trace file format:** After `AcpSession::prompt` opens a trace, it may write a plaintext `Command: …\n` line (from `invocation`), then JSON lines from the agent’s stdout. The file is **not** guaranteed pure JSONL when that prelude exists.
- **`maybe_tee_log`** (`src/agent/ops.rs`) reads the log and prints to stdout unless `--no-tee`. **`strip_trace_invocation_line_for_tee`** (`src/agent/tee_strip.rs`) removes the leading `Command:` line on tee so it is not duplicated after startup `emit_command_line`.
- **Reader:** `TraceChunkCoalescer` and related logic in `src/acp/reader.rs` for chunk coalescing/dedup from `session/update`.

## Tests

- **Node:** Many ACP unit tests use executable Node scripts as mock `agent acp` children; ensure `node` is on `PATH` or spawn/handshake tests fail with stdout closed / initialize errors.
- **Brittle source tests:** Prefer async behavioral tests (e.g. read trace file contents) over `include_str!` substring checks on `src/acp/mod.rs` that break on refactors.

## kiss

Enforces lines-per-file, call counts, duplication, etc. Use `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` so symbols stay visible to kiss. Split modules when limits hit.

## CLI

- `src/cli/args.rs`, `mod.rs`, `shared_opts.rs`; `tee_startup_stdout` gates startup `Command:` + plan echo vs `--no-tee`.
- `prepare_kpop_prompt_store` validates only kpop (+ learn) prompts; full workflow uses `prepare_prompt_store`.

## Workflow (agent)

Reviewer path: after **review** prompt, **sync** workspace review to artifact, **`is_lgtm`** on artifact before **kpop** prompt (`src/agent/ops.rs`).
