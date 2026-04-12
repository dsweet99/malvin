# LLM style — malvin (index)

**TRIGGER** index; detail: `./.llm_style/malvin_tooling.md` (gates, layout, ACP), `./.llm_style/malvin_debugging.md` (debug, search fallbacks).

---

TRIGGER: run checks pre-commit  
ADVICE: From repo root run the suite in `malvin_tooling.md` § Required checks; **`cargo clippy`** must match `.pre-commit-config.yaml` `entry:` verbatim. Rerun mid-task (catches `kiss` limits early); parallelize independent checks.

TRIGGER: kiss check  
ADVICE: `kiss check .` (full project), not bare `kiss`. See `.kissignore`.

TRIGGER: kiss limits  
ADVICE: `lines_per_file` (≈250), `calls_per_function`, or `max_indentation_depth`: split submodules (e.g. `run_timing/report.rs`), extract helpers from hot paths—not unrelated churn. Renames: update `src/coverage_kiss.rs` / `stringify!` when symbols move.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. Pre-commit `admin/check_untracked.sh` fails on untracked `.rs`/`.py`—without `git add`, merge new tests into tracked `tests/*.rs` (see `malvin_tooling.md` § Untracked source files).

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.

TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; confirm code and CLI help match. After fixes, update root `review.md` (no stale “open problems”). ACP: `src/acp/*.inc` (e.g. `ops_body.inc`)—see `malvin_tooling.md`.

TRIGGER: grounding code parity  
ADVICE: When post-run stdout/stderr behavior changes, align `grounding.md` with sources (`src/post_run_hint/report.rs`, `src/run_timing/mod.rs` + `report.rs`, `src/cli/shared_opts.rs`, …). `tests/cli_parity.rs` may `include_str!` those files—see `malvin_tooling.md` § Tests.

TRIGGER: repo-wide string contracts  
ADVICE: Renaming or banning a term: `rg` repo-wide (fragments can hide inside longer words); update `default_prompts/` (agent **pacing** vs thoroughness—not product metrics wording), `.cursorrules`, `_kpop/` logs with code/docs—see `malvin_tooling.md` § Repo-wide string contracts.

TRIGGER: post-run metrics hint  
ADVICE: `src/post_run_hint/report.rs` stderr line + `finish_post_run_hint_then_return` ordering. See `grounding.md` + `malvin_tooling.md` § Post-run metrics hint.

TRIGGER: run timing  
ADVICE: `malvin code` only: optional `AgentClient::timing`; `run_timing.json` + stderr summary **before** post-run metrics hint. Instrument `client_impl.inc` / `ops_body.inc`; finalize from orchestrator—see `grounding.md` + `malvin_tooling.md` § Run timing.

TRIGGER: clippy doc comments  
ADVICE: With `-D warnings`, `clippy::doc_markdown` flags bare identifiers in `//!`/`///`—wrap code-like tokens in backticks (e.g. `CPython`).

TRIGGER: plan.md shipping sync  
ADVICE: When `malvin init`/ACP/models behavior changes, update root `plan.md`; align with `src/cli/init_cmd.rs` and tests—see `malvin_tooling.md`.

TRIGGER: `_malvin` plan  
ADVICE: One-off task specs may live in `_malvin/**/plan.md`—implement when cited; root `plan.md` is working metrics/bootstrap—see `malvin_tooling.md` § `malvin init` + ACP bounded retry.

TRIGGER: KPOP experiment, MBC2, p-creative  
ADVICE: `kpop_acp_prompt.rs`, `ops_body.inc` `run_kpop_flow_once`, outbound counts—see `malvin_tooling.md` § KPOP. Hypothesize → Predict → Falsify; `_malvin/**/_kpop/exp_log_*.md`. Parser/ANSI: `malvin_debugging.md`.

TRIGGER: Rust 2024 rand async  
ADVICE: `gen` is a keyword—use `Uniform` sampling. `Send` across `await`: `StdRng`, not `thread_rng`. Put `use` at module scope. Detail: `malvin_tooling.md` § Rust edition 2024.

TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` + `admin/check_untracked.sh`; bootstrap order matches written `plan.md`; missing `pre-commit`: `tests/init_pre_commit.rs`.

TRIGGER: env_path agent binary  
ADVICE: `src/env_path.rs` `agent_or_cursor_agent_bin()` — same `agent`→`cursor-agent` order as ACP spawn (`ops_body.inc`); use for `malvin models` and any agent CLI resolution.

TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary unit tests cannot import it; for isolated `PATH`, use `tests/*.rs` + `Command::new(env!("CARGO_BIN_EXE_malvin")).env("PATH", …)` (see `init_pre_commit.rs`).

TRIGGER: ACP retry backoff  
ADVICE: `retry_policy.inc` + `backoff_after_agent_failure` in `client_impl.inc`; exhausted messages use `{retries}`—see `malvin_tooling.md` § ACP bounded retry. Upgrade: `Err` only—single `eprintln` at `src/cli/mod.rs`.

TRIGGER: DEFAULT_CLI_MODEL  
ADVICE: `src/cli/shared_opts.rs`; `models_cmd` footer uses `{DEFAULT_CLI_MODEL}`; `default_cli_model_is_composer_2` in `tests/cli_parity.rs` guards drift.

TRIGGER: ACP include layout  
ADVICE: Much of `src/acp/` is `include!` for `kiss`—navigate `.inc` names; **included `.rs` inherit parent `use`** (not a standalone module tree). See `malvin_tooling.md`.

TRIGGER: ACP trace, JSONL, tee  
ADVICE: Traces mix plaintext `Command:` prelude then JSON; `strip_trace_invocation_line_for_tee` + `maybe_tee_log` strip duplicate prelude on tee (`tee_strip_body.inc`, `ops_body.inc`). Reader/coalescing: `reader_inline.inc`, `coalesce.rs`.

TRIGGER: coalesce Unicode scalars  
ADVICE: Track running scalar counts per buffer in verbose/trace coalescing; avoid hot full-buffer `chars().count()` rescans—see `src/acp/coalesce.rs`.

TRIGGER: ACP tests, node  
ADVICE: Mock `agent acp` children often use `#!/usr/bin/env node`; ensure `node` on PATH—`prepend_standard_path_for_child` (`transport/command.rs`) when stripping env. See `malvin_tooling.md` § Tests.

TRIGGER: pre-commit hooks  
ADVICE: `.pre-commit-config.yaml` runs ruff, clippy, kiss, `admin/check_untracked.sh`—not `cargo test`/`pytest`; run full suite manually or in CI.

TRIGGER: CLI, help text  
ADVICE: `src/cli/`: `args.rs`, `mod.rs`, `shared_opts.rs`; `disable_help_subcommand = true`; doc comments become `--help`. Tee: `SharedOpts::tee_startup_stdout`.

TRIGGER: search tools subagents  
ADVICE: If workspace glob/`rg` errors, run `rg`/`find` from repo root (`malvin_debugging.md`). At most 4 parallel subagents for independent exploration; skip for tiny edits.

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; `date` when rules require; if the request matches a TRIGGER keyword, show the single most relevant TRIGGER:/ADVICE: pair.

TRIGGER: all checks must pass, noqa  
ADVICE: Fix all failures everywhere. No `# noqa` except where required for correctness. No test-cheating.
