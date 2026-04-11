# LLM style — malvin (index)

Use **TRIGGER** keywords to recall **ADVICE**. Detail: `./.llm_style/malvin_tooling.md`.

---

TRIGGER: run checks yourself  
ADVICE: From repo root: `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `ruff check .`, `kiss check .`, `pytest -sv tests`. Rerun after substantive edits; parallelize independent checks.

TRIGGER: kiss check  
ADVICE: `kiss check .` (full project), not bare `kiss`. See `.kissignore`.

TRIGGER: kiss line limit  
ADVICE: On `lines_per_file` (≈250), extract submodules—e.g. `agent/tee_strip.rs`, `src/log_paths.rs`, `cli/command_log_tests.rs`—not unrelated churn.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; if review mentions untracked files, tell the user to stage/commit locally.

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.

TRIGGER: review.md plan  
ADVICE: Read `review.md` and `grounding.md` for reviewer work; verify sync → LGTM before kpop in `src/agent/ops.rs`.

TRIGGER: ACP trace, JSONL, tee  
ADVICE: Trace files are mixed plaintext (`Command:` prelude from `invocation` + `AcpSession::prompt`) then JSON from agent stdout—see `prompt` rustdoc. `agent/tee_strip.rs` strips the prelude for `maybe_tee_log` so stdout does not repeat it. Reader coalescing: `src/acp/reader.rs`.

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
ADVICE: `.pre-commit-config.yaml` runs `ruff check .`, `cargo clippy …`, `kiss check .`—not `cargo test`/`pytest`; run full suite manually or in CI.

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
