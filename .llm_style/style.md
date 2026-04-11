# LLM style — malvin (index)

Use **TRIGGER** keywords to recall **ADVICE**. Commands, layout, gates, detail: `./.llm_style/malvin_tooling.md`.

---

TRIGGER: run checks yourself  
ADVICE: From repo root: `ruff check .`, `kiss check .`, `pytest -sv tests`, `cargo test`, and **`cargo clippy` exactly as in `.pre-commit-config.yaml`** (stricter than `-D warnings` alone). Rerun after substantive edits; parallelize independent checks.

TRIGGER: pre-commit parity  
ADVICE: Match the `cargo-clippy` `entry:` string in `.pre-commit-config.yaml` when reproducing CI; see verbatim block in `malvin_tooling.md`.

TRIGGER: kiss check  
ADVICE: `kiss check .` (full project), not bare `kiss`. See `.kissignore`.

TRIGGER: kiss line limit  
ADVICE: On `lines_per_file` (≈250), extract submodules (e.g. `src/acp/coalesce.rs`, `src/log_paths.rs`, `cli/command_log_tests.rs`)—not unrelated churn.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; if review mentions untracked files, tell the user to stage/commit locally.

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.

TRIGGER: review.md plan  
ADVICE: Read `review.md` and `grounding.md` for reviewer work; verify sync → LGTM before kpop (logic in `src/acp/` includes such as `ops_body.inc`, not only a legacy `src/agent/ops.rs` path).

TRIGGER: ACP include layout  
ADVICE: Much of `src/acp/` is built with `include!` for `kiss` limits—navigate by `.inc` / file names; see `malvin_tooling.md`.

TRIGGER: ACP trace, JSONL, tee  
ADVICE: Traces mix plaintext `Command:` prelude then JSON; `strip_trace_invocation_line_for_tee` + `maybe_tee_log` strip duplicate prelude on tee (`tee_strip_body.inc`, `ops_body.inc`). Reader/coalescing: `reader_inline.inc`, `coalesce.rs`.

TRIGGER: coalesce Unicode scalars  
ADVICE: Track running scalar counts per buffer in verbose/trace coalescing; avoid hot full-buffer `chars().count()` rescans—see `src/acp/coalesce.rs`.

TRIGGER: ACP tests, node  
ADVICE: Many ACP tests spawn `#!/usr/bin/env node` mocks; `node` must be on PATH or handshake tests fail.

TRIGGER: orchestrator stems  
ADVICE: Use `prompt_md_stem` / `strip_suffix(".md")` in `src/orchestrator/`; do not slice with `len()-3`.

TRIGGER: prompts include_str  
ADVICE: Defaults in `default_prompts/`; `src/prompts/mod.rs` embeds via `../../default_prompts/...`.

TRIGGER: coverage_kiss stringify  
ADVICE: Renames may need `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` tests updated.

TRIGGER: MSRV edition  
ADVICE: `edition = "2024"`, `rust-version = "1.85"` in `Cargo.toml`; mention in `README.md` if documenting toolchain.

TRIGGER: pre-commit hooks  
ADVICE: `.pre-commit-config.yaml` runs ruff, clippy, kiss, `admin/check_untracked.sh`—not `cargo test`/`pytest`; run full suite manually or in CI.

TRIGGER: CLI, help text  
ADVICE: `src/cli/`: `args.rs`, `mod.rs`, `shared_opts.rs`; `disable_help_subcommand = true`; doc comments become `--help`. Tee: `SharedOpts::tee_startup_stdout`.

TRIGGER: verify before implementing  
ADVICE: Read existing code; `review.md` items may already be fixed on disk.

TRIGGER: parallel subagents  
ADVICE: At most 4 parallel subagents for independent exploration; skip for tiny edits.

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; optional `date` when rules ask.

TRIGGER: all checks must pass, noqa  
ADVICE: Fix all failures everywhere. No `# noqa` except where required for correctness. No test-cheating.

TRIGGER: TRIGGER / ADVICE  
ADVICE: After a user request, if TRIGGER words match, show the single most relevant TRIGGER:/ADVICE: pair.
