# Malvin repo — tooling and quality gates

## Required checks (run from repo root)

- `ruff check .`
- `pytest -q`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings` (stricter than default; pre-commit also passes many `-W clippy::*` and allows some `-A` overrides)
- `kiss check .` — **not** bare `kiss` (which only prints help)

## Constraints

- **Do not edit** `.kissconfig` (user rule).
- **Rust 2024** + `rust-version = "1.85"` in `Cargo.toml`; document MSRV in `README.md` if extending docs.
- **`.kissignore`** may exclude paths (e.g. large vendored `src/acp/`) from kiss metrics; still run full `kiss check .` on what remains.

## Pre-commit (`.pre-commit-config.yaml`)

Runs `ruff`, `pytest`, `cargo clippy`, `cargo test`, `kiss check .` — contributors need those binaries or must skip hooks.

## Layout pointers

- Library: `src/lib.rs`; binary: `src/main.rs`.
- ACP: `src/acp/`; agent orchestration: `src/agent/` (`client`, `ops`, `pair`).
- Workflow: `src/orchestrator/` + `src/review_sync.rs` (shared `is_lgtm` / `sync_review_file`).
- Prompts + `include_str!`: `src/prompts/mod.rs` → `../../default_prompts/`; ship `default_prompts/` in repo.
- Run dirs: `_malvin/<stamp>/` (often gitignored).

## Review workflow

After the **review** `session/prompt`, **sync** workspace `review.md` to the run artifact, then **`is_lgtm` on the artifact** before any **kpop** prompt (`src/agent/ops.rs`). Coder session is long-lived; reviewer session does review → (if not LGTM) kpop in one ACP session when kpop runs.

## kiss / structure

kiss enforces file length, argument counts, call counts, duplication — large files may need splitting (e.g. `prompts/` as `mod.rs` + `tests.rs`). `src/coverage_kiss.rs` uses `stringify!` so kiss sees symbols; keep in sync when renaming APIs.
