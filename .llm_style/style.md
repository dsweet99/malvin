# LLM style — malvin (index)

When the project `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail: `./.llm_style/malvin_tooling.md` (gates, layout, ACP, CLI, review sync, `/proc` child health), `./.llm_style/malvin_debugging.md` (KPOP, search fallbacks).

---

TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks (Rust + **`pytest -sv tests`**); **`cargo clippy`** must match `.pre-commit-config.yaml` `entry:` verbatim. Fix every failure; no `# noqa` except for correctness; no test-cheating. Rerun mid-task (kiss limits); parallelize independent checks. **`clippy::double_must_use`:** do not add `#[must_use]` on `fn` that already returns a `#[must_use]` type (e.g. `Result`).
TRIGGER: kiss check and limits  
ADVICE: `kiss check .` (full project), not bare `kiss`—see `.kissignore`. Limits: `lines_per_file` (e.g. ~250 `src/prompts/mod.rs` → `prompts/template.rs`), `calls_per_function`, `max_indentation_depth`, **duplication**, **concrete_types_per_file**: split modules, extract shared helpers, move CLI arg structs to dedicated files—not unrelated churn. Update `src/coverage_kiss.rs` / `stringify!` when symbols move. See `malvin_tooling.md` § kiss.
TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.
TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. Pre-commit `admin/check_untracked.sh` fails on untracked `.rs`/`.py`—without `git add`, merge new tests into tracked `tests/*.rs` (e.g. `cli_parity.rs`); see `malvin_tooling.md` § Untracked.
TRIGGER: cli mod sibling file  
ADVICE: `src/cli/mod.rs` `mod name;` requires `src/cli/name.rs`; ship in the same change—`malvin_tooling.md` § CLI.
TRIGGER: child health ACP silence  
ADVICE: `src/child_health/` (`linux`/`macos`/`other`); `process_absent` vs `cannot_sample`; `counters_trusted`; `rpc_wait_response` races JSON-RPC `oneshot` with `evaluate_after_acp_silence`. See `malvin_tooling.md` § Child health + ACP silence.
TRIGGER: voluntary_ctxt parse  
ADVICE: Linux `parse_status_voluntary_ctxt`: after `strip_prefix`, use **`rest.trim().parse()`**—`trim_start()` leaves trailing `\r` and breaks `u64` parse. See `malvin_tooling.md` § Child health.
TRIGGER: malvin binary crate  
ADVICE: `src/cli/` is binary-only—not the `malvin` library—so `pub(crate)` on `AgentClient` fields is not visible there; use public lib methods or keep access in lib modules. Private `src/cli/*.rs` submodules: `pub fn` not `pub(crate) fn` when `clippy::redundant_pub_crate` fires. See `malvin_tooling.md` § Crate layout.
TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).
TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.
TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; update root `review.md` after fixes (no stale “open problems”). Review sync API + paths: `malvin_tooling.md` § Review sync + `review.md`.
TRIGGER: grounding code parity  
ADVICE: When run-timing, tee, or post-run stdout/stderr behavior changes, align `grounding.md` with sources (`run_timing/`, `src/acp/`, …). Helpers that only merge `Result`s after I/O must not read as reordering streams (`kpop_flow.rs`). `tests/cli_parity.rs` may `include_str!` implementation files and `ops_body.inc` (e.g. reviewer pair order test)—see `malvin_tooling.md` § Tests + Review sync.
TRIGGER: repo-wide string contracts  
ADVICE: Renaming or banning a term: `rg` repo-wide (fragments can hide inside longer words); update `default_prompts/`, `.cursorrules`, `_kpop/` logs—see `malvin_tooling.md` § Repo-wide string contracts.
TRIGGER: post-run metrics hint  
ADVICE: Optional stderr line after run timing: stable “not measured” copy only (no gross/net/repo-tree metering). Message must not contain `"git"` (`tests/cli_parity.rs`). Ordering: `grounding.md` + `malvin_tooling.md` § Post-run metrics hint.
TRIGGER: run timing  
ADVICE: `malvin code` / `malvin kpop` / `malvin do`: `run_timing.json` + one stdout `TIMING:` line after the workflow body (**before** stderr post-run hint)—**same `serde_json::Value`** drives disk + stdout (`report.rs`); `attach_new_run_timing` / `attach_run_timing_for_session` for all three; `emit_run_timing_after_acp` (`timing_merge.rs`) for `do`/`kpop` finalize. If timing I/O and workflow/ACP both fail, prefer the primary error—see `grounding.md` + `malvin_tooling.md` § Run timing.
TRIGGER: plan.md root vs `_malvin`  
ADVICE: Root `plan.md` vs shipped init/ACP/models (`init_cmd.rs`, tests); one-off `_malvin/**/plan.md` when cited—`malvin_tooling.md` § `malvin init` + ACP bounded retry.
TRIGGER: KPOP experiment, MBC2, p-creative  
ADVICE: `kpop_acp_prompt.rs`, `ops_body.inc` `run_kpop_flow_once`—`malvin_tooling.md` § KPOP. `malvin models` parser/ANSI: `malvin_debugging.md`.
TRIGGER: KPOP HPF log  
ADVICE: **Hypothesize → Predict → Falsify**; session artifact path; **`mkdir -p`** before write—`malvin_debugging.md` § KPOP. IDE search I/O → shell `rg` from repo root.
TRIGGER: Rust 2024 rand async  
ADVICE: `gen` is a keyword—use `Uniform` sampling. `Send` across `await`: `StdRng`, not `thread_rng`. Detail: `malvin_tooling.md` § Rust edition 2024.
TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` + `admin/check_untracked.sh`; `tests/init_pre_commit.rs` when `pre-commit` missing.
TRIGGER: env_path agent binary  
ADVICE: `src/env_path.rs` `agent_or_cursor_agent_bin()` — same `agent`→`cursor-agent` order as ACP spawn (`ops_body.inc`).
TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary tests use `tests/*.rs` + `env!("CARGO_BIN_EXE_malvin")` (see `init_pre_commit.rs`).
TRIGGER: ACP retry backoff  
ADVICE: `retry_policy.inc`, `client_impl.inc`, `retry_policy_tests` in `agent_bundle.inc`—narrow `contains` phrases; guard **`timeout_*`** false positives—`malvin_tooling.md` § ACP bounded retry.
TRIGGER: DEFAULT_CLI_MODEL  
ADVICE: `src/cli/shared_opts.rs`; `models_cmd` footer `{DEFAULT_CLI_MODEL}`; `default_cli_model_is_composer_2` in `tests/cli_parity.rs`.
TRIGGER: ACP include layout  
ADVICE: Much of `src/acp/` is `include!` for `kiss`—navigate `.inc` names; **included `.rs` inherit parent `use`**. See `malvin_tooling.md`.
TRIGGER: ACP trace, JSONL, tee  
ADVICE: Live tee: stdout reader (`trace_file_write_line`, `coalesce.rs`, `reader_inline.inc`). Test-only `strip_trace_invocation_line_for_tee` (`tee_strip_tests.inc`); **no** post-prompt file tee stub—`malvin_tooling.md` § ACP traces.
TRIGGER: ACP tests, node  
ADVICE: Mock `agent acp` children often `#!/usr/bin/env node`; `prepend_standard_path_for_child` (`transport/command.rs`). See `malvin_tooling.md` § Tests.
TRIGGER: pre-commit hooks  
ADVICE: `.pre-commit-config.yaml`: ruff, clippy, kiss, `admin/check_untracked.sh`—**not** `cargo test`/`pytest`; run full suite manually or in CI.
TRIGGER: search tools subagents  
ADVICE: Workspace search I/O errors → shell `rg` from repo root (`malvin_debugging.md`). Merge-marker sweeps: limit to text globs / exclude `target/`—`malvin_tooling.md` § Merge markers, `_malvin` plans, green tree. ≤4 parallel subagents; skip for tiny edits.
TRIGGER: full suite scope  
ADVICE: When the user demands all checks/tests green on **all** files, fix repo-wide failures—no “pre-existing” hand-waving; see `malvin_tooling.md` § green tree no excuses.
TRIGGER: CLI async timing finalize  
ADVICE: After `await` ACP work, call sync `emit_run_timing_after_acp` (`src/cli/timing_merge.rs`)—avoid async helpers taking `FnOnce(&mut AgentClient) -> Fut` (lifetime errors with `&mut` + returned `Future`). See `malvin_tooling.md` § Run timing.
TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; matching TRIGGER → show one TRIGGER:/ADVICE: pair. Prefer **running commands** over instruction-only when the user expects work.
