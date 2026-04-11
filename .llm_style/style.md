# LLM style — malvin (index)

Use **TRIGGER** keywords/phrases to recall **ADVICE**. Detail: `./.llm_style/malvin_tooling.md`.

---

TRIGGER: run checks yourself  
ADVICE: Execute `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `ruff check`, `kiss check` from repo root; rerun after substantive edits; parallelize independent checks. (No Python code; pytest collects 0.)

TRIGGER: kiss check  
ADVICE: Use `kiss check .` (full project), not bare `kiss`. See `.kissignore` for excluded paths.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig` in this repo.

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (with predictions/test/confounders when useful). Reserve Claim for statements with cited evidence (code, logs, metrics).

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match existing naming, modules, and comment level; avoid drive-by refactors.

TRIGGER: review.md plan  
ADVICE: Read `review.md` and `grounding.md` when addressing reviewer parity; verify workflow order (sync → LGTM before kpop) in `src/agent/ops.rs`.

TRIGGER: orchestrator stems  
ADVICE: Use `prompt_md_stem` / `strip_suffix(".md")` for log name stems in `src/orchestrator/`; do not slice with `len()-3`.

TRIGGER: prompts include_str  
ADVICE: Default prompts live in `default_prompts/`; `src/prompts/mod.rs` uses `../../default_prompts/...` paths.

TRIGGER: module too long  
ADVICE: If kiss fails on lines/calls/args/duplication, split into submodules (see `agent/`, `orchestrator/`, `prompts/` patterns) or extract helpers/structs.

TRIGGER: coverage_kiss stringify  
ADVICE: Renaming public APIs may require updating `src/coverage_kiss.rs` and `kiss_refs` tests so kiss still sees coverage.

TRIGGER: MSRV edition  
ADVICE: Crate uses `edition = "2024"` and `rust-version = "1.85"` in `Cargo.toml`; mention in `README.md` if documenting toolchain.

TRIGGER: pre-commit hooks  
ADVICE: `.pre-commit-config.yaml` expects `ruff`, `cargo`, `kiss` on PATH; pytest is legacy (no Python code).

TRIGGER: CLI structure, subcommands, help text  
ADVICE: CLI lives in `src/cli/`: `args.rs` (Cli, Commands), `mod.rs` (dispatch), `shared_opts.rs`. Use `disable_help_subcommand = true` on Cli to hide `help` subcommand. Doc comments become help text.

TRIGGER: ACP trace, log files, tee output  
ADVICE: `src/acp/reader.rs` has `TraceChunkCoalescer` for plain-text log writing; `src/agent/ops.rs` `maybe_tee_log` handles tee. Logs contain deduplicated text from `agent_message_chunk`/`agent_thought_chunk`.

TRIGGER: verify before implementing  
ADVICE: Read existing code structure before making changes; plans may already be implemented. Avoid wasted effort.

TRIGGER: parallel subagents  
ADVICE: Use at most 4 parallel subagents for independent exploration; avoid for tiny single-file edits.

TRIGGER: user communication  
ADVICE: Prefer precise prose, full URLs/paths, and ```line:line:path``` code citations; avoid filler closings; keep final answer proportional to task size.

TRIGGER: all checks must pass, noqa  
ADVICE: No excuses for "pre-existing" failures. Fix ALL files, not just modified ones. No `# noqa` additions (except for correct functioning). No test-cheating. Be tenacious.

TRIGGER: TRIGGER / ADVICE  
ADVICE: After user requests, if this file's TRIGGER words match, show the single most relevant TRIGGER:/ADVICE: pair to the user.
