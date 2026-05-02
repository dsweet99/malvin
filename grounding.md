# Malvin — project contract

## Purpose

- **malvin** is a non-interactive CLI that drives a coding agent over **Cursor ACP** (stdio JSON-RPC), for workflows like one-shot `do`, plan-driven `code`, repo hygiene `tidy`, Popper-style `kpop`, and beta `sync` / `ground`.
- Crate: **Rust 2024**, `unsafe_code = "deny"` (see `Cargo.toml`). Library + `malvin` binary.

## Languages & tools (this tree)

- **Primary:** Rust (`cargo`, `clippy`, `test`). **kiss** enforces structural limits on Rust (and would on Python if present under the workspace walk rules in `src/repo_gates.rs`).
- **Conditional:** If the workspace contains `.py` files (excluding skipped dirs such as `target` and dot-directories per `visit_source_files` in `src/repo_gates.rs`), built-in gates also include `ruff check .` and, when pytest-style test modules exist, `pytest -sv tests`.
- **This repository** (sources outside `target/`): **Rust only** for built-in gate lines; no `ruff` / `pytest` unless you add Python sources or `.malvin_checks` lines.

## CLI commands (stable surface)

Order matches `malvin --help` (`Commands` in `src/cli/args.rs`):

| Command | Role |
|--------|------|
| `init` | Prepare a workspace (repo). |
| `do` | Single request / payload. |
| `code` | Implement → reviews → learn loop from `request` / `@file` → `_malvin/.../plan.md`. |
| `kpop` | Hypothesis-driven investigation; `request` → `_malvin/.../request.md`; exp logs under `_kpop/`. |
| `tidy` | Bring checks to green via tidy prompt + gates. |
| `models` | List / parse model metadata. |
| `sync` *(beta)* | Align implementation with this contract file. |
| `ground` *(beta)* | Author or refine **`./grounding.md`** until `check_sync` review is **LGTM**; **does not target application source** for edits (see `check_sync` / grounding prompts). |

- **`kiss` on PATH** is required before **`malvin code`** and **`malvin tidy`** start (`require_kiss_for_cli_command` in `src/cli/entrypoint.rs`). Other subcommands do not perform that preflight.

## Artifacts & paths

- Each run uses **`{work_dir}/_malvin/<run_id>/`**: e.g. `plan.md` or `request.md`, `review.md`, `result.md`, phase logs, optional `_kpop/exp_log_*.md`.
- Workspace review file: **`review.md`** under the working tree where applicable (`RunArtifacts` / artifact helpers in `src/artifacts/`).

## Review & abort semantics

- **LGTM:** After trim, body must be exactly `LGTM` (leading UTF-8 BOM allowed); other text is not accepted (`is_lgtm_str` in `src/review_sync/`).
- **`ground`:** Up to **5** `check_sync` attempts (`GROUND_MAX_LOOPS` in `src/cli/ground_cmd.rs`). Exhaustion error: `Did not receive LGTM for check_sync.md within max loops.`
- **`result.md`:** A line starting with **`ABORT:`** fails the workflow after the phase that reads it (`ground_cmd.rs` and shared abort handling).

## Quality gates (workspace)

**Built-in command list** comes from `repo_gates::gate_command_lines` (`src/repo_gates.rs`), in order:

1. `kiss check`
2. `ruff check .` — if any `.py` discovered (see above)
3. `pytest -sv tests` — if pytest-style `test_*.py` / `*_test.py` modules exist
4. If `Cargo.toml` exists:
   - `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc`
   - `cargo test`
5. **Append** non-empty trimmed lines from **`.malvin_checks`** if that file exists (Malvin **never** creates or edits `.malvin_checks`).

**Workspace gate runner** (`run_repo_workspace_gates` in `src/cli/repo_checks/workspace.rs`):

- Runs **`kiss clamp`** during prepare **only when** `.kissconfig` is missing **and** the tree has source-like files (`kiss_clamp::has_source_files`). If `.kissconfig` is present, Malvin does not run `kiss clamp` for that reason.
- **Does not** run **`pre-commit`**; tests assert the gate log never contains `pre-commit run --all-files` even when `.pre-commit-config.yaml` exists.

**When full gates run (summary):**

- **`do`:** Default path runs only **`kiss_clamp::ensure_kiss_clamp_if_needed`** (clamp when `.kissconfig` is missing and source-like files exist — **no** `prepare_repo_workspace` / `warn_kissconfig_test_coverage_if_needed`, **no** `kiss check` / `cargo clippy` / `cargo test`). With **`--repo-gates`**, runs **`run_repo_workspace_gates`** first (full prepare + all `gate_command_lines`; `RepoGateOutput::Stderr` in `src/cli/do_flow.rs`). There is no pre-summary gate pass after the single `do` ACP prompt.
- **`code` / `sync`:** gates at workflow start; then after implement/review/learn path and **before** the summary ACP phase, **`run_pre_summary_repo_gates_with_tidy_retry`** may run `tidy`-on-failure then re-gates (`src/cli/mid_session_gates.rs`, `Orchestrator::run_with_pre_summary_gap` in `src/orchestrator/mod.rs`).
- **`tidy`:** startup uses **prepare only** (clamp path + warnings, no quality shell commands); **after** the tidy (and optional learn) body, the same pre-summary gate + tidy-retry block runs, then summary (`src/cli/tidy_flow/helpers.rs`).
- **`ground`:** runs **`run_repo_workspace_gates`** before the grounding ACP session (`src/cli/ground_cmd.rs`).
- **`kpop`:** does **not** call `run_repo_workspace_gates`; templates still receive **`quality_gates`** markdown from `prompt_quality_gates_markdown` for the agent to follow manually (`src/cli/kpop_flow.rs` — no `run_repo_workspace_gates` in that module).

**Optional extra gates:** add shell lines to **`.malvin_checks`** at repo root (one command per non-empty line).

## Hard constraints & non-goals

- **`./grounding.md`:** Only the **`malvin ground`** workflow should create or edit this file (via bundled prompts `write_grounding.md` / `improve_grounding.md`). Other commands must not rewrite it as part of their normal path.
- **`.kissconfig`:** Treat as project-owned configuration; do not modify it from Malvin features unless explicitly intended product behavior (operators edit it directly).
- Malvin is **not** a generic `pre-commit` runner; use `.malvin_checks` if you need extra hooks.

## Required checks (operator / agent expectation)

For a Rust workspace matching this repo, a green tree means at least:

- `kiss check`
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc`
- `cargo test`

Plus any lines in **`.malvin_checks`** if present. Run **`kiss rules`** before large edits to avoid structural violations.

## Operational assumptions

- **ACP:** Requires a working Cursor ACP agent binary and environment (e.g. `CURSOR_AGENT_API_KEY` where applicable). Failures surface as CLI / workflow errors, not interactive prompts.
- **Paths:** Commands run relative to the chosen workspace directory; artifact paths are resolved under `_malvin/<run_id>/` there.
- **Beta:** `sync` and `ground` behavior may evolve; this file should track what the code in `src/cli/sync_flow.rs`, `src/cli/ground_cmd.rs`, and orchestrator modules actually do.
