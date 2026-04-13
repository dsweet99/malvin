# LLM style — malvin (index)

When the project `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail: `./.llm_style/malvin_tooling.md` (gates, layout, ACP, child health), `./.llm_style/malvin_debugging.md` (debug, search fallbacks).

---

TRIGGER: all checks pre-commit  
ADVICE: Run full suite in `malvin_tooling.md` § Required checks; **`cargo clippy`** must match `.pre-commit-config.yaml` `entry:` verbatim. Fix every failure (no “pre-existing”); no `# noqa` except for correctness; no test-cheating. Rerun mid-task (kiss limits); parallelize independent checks.

TRIGGER: kiss check  
ADVICE: `kiss check .` (full project), not bare `kiss`. See `.kissignore`.

TRIGGER: kiss limits  
ADVICE: `lines_per_file` (≈250), `calls_per_function`, or `max_indentation_depth`: split submodules (e.g. `run_timing/report.rs`, `child_health/tests.rs`), extract helpers—not unrelated churn. Renames: update `src/coverage_kiss.rs` / `stringify!` when symbols move.

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. Pre-commit `admin/check_untracked.sh` fails on untracked `.rs`/`.py`—without `git add`, merge new tests into tracked `tests/*.rs` (see `malvin_tooling.md` § Untracked). New lib modules under `src/` (e.g. `child_health/`) must be tracked **with** `lib.rs` / `Cargo.toml` / `Cargo.lock` for reproducible clones.

TRIGGER: child health ACP silence  
ADVICE: `src/child_health/` (`linux`/`macos`/`other`); `process_absent` vs `cannot_sample`; `counters_trusted`; `rpc_wait_response` races JSON-RPC `oneshot` with `evaluate_after_acp_silence`. See `malvin_tooling.md` § Child health + ACP silence.

TRIGGER: malvin binary crate  
ADVICE: `src/cli/` is binary-only—not the `malvin` library—so `pub(crate)` on `AgentClient` fields (e.g. `timing`) is not visible there; use public lib methods (`attach_run_timing_for_session`, …) or keep access in lib modules. See `malvin_tooling.md` § Crate layout.

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.

TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; confirm code and CLI help match. After fixes, update root `review.md` (no stale “open problems”). ACP: `src/acp/*.inc` (e.g. `ops_body.inc`)—see `malvin_tooling.md`.

TRIGGER: grounding code parity  
ADVICE: When post-run stdout/stderr behavior changes, align `grounding.md` with sources (`post_run_hint/`, `run_timing/`, …). Helpers that only merge `Result`s after I/O must not read as reordering streams (`kpop_flow.rs`). `tests/cli_parity.rs` may `include_str!`—see `malvin_tooling.md` § Tests.

TRIGGER: repo-wide string contracts  
ADVICE: Renaming or banning a term: `rg` repo-wide (fragments can hide inside longer words); update `default_prompts/` (agent **pacing** vs thoroughness—not product metrics wording), `.cursorrules`, `_kpop/` logs with code/docs—see `malvin_tooling.md` § Repo-wide string contracts.

TRIGGER: post-run metrics hint  
ADVICE: **`src/post_run_hint/`** (`report.rs`): stable “not measured” stderr only; **gross/net/git-tree metering removed** (see `mod.rs`). Message must not contain `"git"` (`tests/cli_parity.rs`). `finish_post_run_hint_then_return` ordering—`grounding.md` + `malvin_tooling.md` § Post-run metrics hint.

TRIGGER: run timing  
ADVICE: `malvin code` / `malvin kpop`: `run_timing.json` + stdout summary **before** stderr post-run hint; record in `client_impl.inc` / `ops_body.inc`; `attach_run_timing_for_session` / finalize from orchestrator or KPOP. If timing I/O and workflow/ACP both fail, prefer the primary error—see `grounding.md` + `malvin_tooling.md` § Run timing.

TRIGGER: plan.md root vs `_malvin`  
ADVICE: Root `plan.md`: sync with shipped `malvin init`/ACP/models (`src/cli/init_cmd.rs`, tests). One-off task specs: `_malvin/**/plan.md` when cited. See `malvin_tooling.md` § `malvin init` + ACP bounded retry.

TRIGGER: KPOP experiment, MBC2, p-creative  
ADVICE: `kpop_acp_prompt.rs`, `ops_body.inc` `run_kpop_flow_once`, outbound counts—`malvin_tooling.md` § KPOP. `malvin models` parser/ANSI: `malvin_debugging.md`.

TRIGGER: KPOP HPF log  
ADVICE: **Hypothesize → Predict → Falsify**; restate problem first; optional **hypothesis budget**; session artifact path and incremental logging, **`mkdir -p`** before write, closing summary—`malvin_debugging.md` § KPOP. IDE search I/O errors → shell `rg` from repo root—same file.

TRIGGER: Rust 2024 rand async  
ADVICE: `gen` is a keyword—use `Uniform` sampling. `Send` across `await`: `StdRng`, not `thread_rng`. Put `use` at module scope. Detail: `malvin_tooling.md` § Rust edition 2024.

TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` + `admin/check_untracked.sh`; bootstrap order matches written `plan.md`; missing `pre-commit`: `tests/init_pre_commit.rs`.

TRIGGER: env_path agent binary  
ADVICE: `src/env_path.rs` `agent_or_cursor_agent_bin()` — same `agent`→`cursor-agent` order as ACP spawn (`ops_body.inc`); use for `malvin models` and any agent CLI resolution.

TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary unit tests cannot import it; for isolated `PATH`, use `tests/*.rs` + `Command::new(env!("CARGO_BIN_EXE_malvin")).env("PATH", …)` (see `init_pre_commit.rs`).

TRIGGER: ACP retry backoff  
ADVICE: Retriable substrings + `plan_agent_retry` in `retry_policy.inc`; backoff loop in `client_impl.inc`; policy tests in `agent_bundle.inc` (`retry_policy_tests`). Add narrow `contains` phrases; guard **`timeout_*`** validation false positives—see `malvin_tooling.md` § ACP bounded retry. Upgrade: `Err` only—single `eprintln` at `src/cli/mod.rs`.

TRIGGER: DEFAULT_CLI_MODEL  
ADVICE: `src/cli/shared_opts.rs`; `models_cmd` footer uses `{DEFAULT_CLI_MODEL}`; `default_cli_model_is_composer_2` in `tests/cli_parity.rs` guards drift.

TRIGGER: ACP include layout  
ADVICE: Much of `src/acp/` is `include!` for `kiss`—navigate `.inc` names; **included `.rs` inherit parent `use`** (not a standalone module tree). See `malvin_tooling.md`.

TRIGGER: ACP trace, JSONL, tee  
ADVICE: Traces mix plaintext `Command:` prelude then JSON; `strip_trace_invocation_line_for_tee` + `maybe_tee_log` strip duplicate prelude on tee (`tee_strip_body.inc`, `ops_body.inc`). Reader/coalescing: `reader_inline.inc`, `coalesce.rs` (Unicode scalar counts per buffer—see `malvin_tooling.md` § ACP traces).

TRIGGER: ACP tests, node  
ADVICE: Mock `agent acp` children often use `#!/usr/bin/env node`; ensure `node` on PATH—`prepend_standard_path_for_child` (`transport/command.rs`) when stripping env. See `malvin_tooling.md` § Tests.

TRIGGER: pre-commit hooks  
ADVICE: `.pre-commit-config.yaml` runs ruff, clippy, kiss, `admin/check_untracked.sh`—not `cargo test`/`pytest`; run full suite manually or in CI.

TRIGGER: CLI, help text  
ADVICE: `src/cli/`: `args.rs`, `mod.rs`, `shared_opts.rs`; `disable_help_subcommand = true`; doc comments become `--help`. Tee: `SharedOpts::tee_startup_stdout`.

TRIGGER: search tools subagents  
ADVICE: Workspace **glob/search** I/O (e.g. `rg: IO error`) → shell `rg`/`find` from repo root (`malvin_debugging.md`). ≤4 parallel subagents; skip for tiny edits.

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; `date` when rules require; matching TRIGGER → show one TRIGGER:/ADVICE: pair. Prefer **running commands** over instruction-only replies when the user expects work (shell `rg` if IDE search fails).
