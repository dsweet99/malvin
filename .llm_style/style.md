# LLM style — malvin (index)

When `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail: `./.llm_style/malvin_tooling.md` (gates, layout, ACP, **`src/output/`** tee + prefixed lines, CLI, docs parity, child health, LiteLLM), `./.llm_style/malvin_debugging.md` (KPOP, root **`plan.md`** backlog, search fallbacks).

---

TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks (Rust + **`pytest -sv tests`** with **`PYTHONPATH=.`** when tests import the repo); **`cargo clippy`** must match `.pre-commit-config.yaml` `entry:` verbatim. Fix every failure; no `# noqa` except for correctness; no test-cheating. Rerun mid-task (kiss limits); parallelize independent checks. **`clippy::double_must_use`:** do not add `#[must_use]` on `fn` that already returns a `#[must_use]` type (e.g. `Result`).
TRIGGER: kiss check and limits  
ADVICE: `kiss check .` (full project), not bare `kiss`—see `.kissignore`. Limits: `lines_per_file`, `calls_per_function`, `max_indentation_depth`, **duplication**, **concrete_types_per_file**: split modules, extract helpers—not unrelated churn. **`src/cli/args.rs`** is often at the type cap—fold new flattened CLI structs into **`shared_opts.rs`** (e.g. `GlobalOpts`) instead of only growing `args.rs`. Update `src/coverage_kiss.rs` / `stringify!` when symbols move; add **`#[cfg(test)]`** `stringify!` / smoke in the **same file** when kiss flags a module (see `malvin_tooling.md` § kiss + § Prefixed log lines).
TRIGGER: clippy doc first paragraph  
ADVICE: **`clippy::too_long_first_doc_paragraph`**: keep the opening **`///`** paragraph short; put detail in following paragraphs (pre-commit uses `-D warnings`).
TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.
TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. Pre-commit `admin/check_untracked.sh` fails on untracked `.rs`/`.py`—without `git add`, merge new tests into tracked `tests/*.rs` (e.g. `cli_parity.rs`); see `malvin_tooling.md` § Untracked.
TRIGGER: clap help command order  
ADVICE: `malvin --help` lists subcommands in **`src/cli/args.rs`** `Commands` enum declaration order—reorder variants to change the usage list; see `malvin_tooling.md` § CLI.
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
ADVICE: Read `review.md` + `grounding.md`; update root `review.md` after fixes (no stale “open problems”). Verify claims vs `src/` + tests—**reviewer bullets can lag** already-correct docs/code; resolve by updating `review.md` and adding **`tests/cli_parity.rs`** guards when useful. Review sync API + paths: `malvin_tooling.md` § Review sync + `review.md`.
TRIGGER: malvin do --raw  
ADVICE: `do_flow.rs` passes `skip_repo_style: do_args.raw` into `AgentClient::run_coder_prompt`; `compose_coder_prompt_for_session` in `client_impl.inc` skips `.style/main.md` on the first coder turn when true. Orchestrator passes `false`. Align `grounding.md`; regress `tests/cli_parity.rs` + `compose_coder_prompt_tests` (`agent_bundle.inc`). Detail: `malvin_tooling.md` § CLI + coder prompt compose.
TRIGGER: grounding code parity  
ADVICE: When run-timing, tee, or workflow stdout/stderr behavior changes, align **`grounding.md`** with sources (`run_timing/`, `src/acp/`, `src/output/`, …). **`.llm_style/*.md`** must not describe removed or nonexistent paths—regressions guarded in `tests/cli_parity.rs` (`include_str!` on `grounding.md`, `.llm_style/`; e.g. obsolete `src/artifacts.rs` vs `src/artifacts/`—`malvin_tooling.md` § Tests). Helpers that only merge `Result`s after I/O must not read as reordering streams (`kpop_flow.rs`). See `malvin_tooling.md` § Tests + docs parity.
TRIGGER: stdout stderr log header  
ADVICE: Route through **`src/output/mod.rs`** (`print_stdout_line`, `print_stderr_line`, `format_line`, …). **ACP tee** ANSI + direction: **`src/output/acp_tee.rs`** (`AcpTeeDirection`, `print_stdout_acp_tee_line`)—outbound vs inbound colors; wire points **`session_trace.rs`** / **`coalesce.rs`**. **Logical** text: `YYYYMMDD.HHMMSS.mmm:[who]: …` with `[who]` padded/truncated to **`LOG_TAG_INNER_WIDTH`** Unicode scalars. **Disk** and **stderr** plain `format_line` (no ANSI). Default stdout prefix coloring: dim timestamp + cyan `who` unless ACP tee path. Document in **`grounding.md`**. Detail: `malvin_tooling.md` § Prefixed log lines.
TRIGGER: learn ACP tee  
ADVICE: Outbound `>learn`: omit stdout echo when tee on (`acp_tee_echo_outgoing_prompt_lines`, `src/acp/session_trace.rs`); disk trace keeps full prompt text. Inbound `<learn`: at most one stdout `[learning...]` (`prompt_stdout_replacement`, `trace_tee_stdout_line` in `session.rs` / `coalesce.rs`). Do not mutate on-disk trace content for redaction; unit-test pure tee helpers beside `kiss_stringify_*` in the same file—`malvin_tooling.md` § ACP learn tee.
TRIGGER: source-shape regression tests  
ADVICE: After changing ACP prompt signatures or include-body call shapes, check string-based tests like `tests/review_ops_order.rs` and docs-parity tests that `include_str!` source files—they may need updates even when runtime behavior is correct.
TRIGGER: repo-wide string contracts  
ADVICE: Renaming or banning a term: `rg` repo-wide (fragments can hide inside longer words); update `default_prompts/`, `.cursorrules`, `_kpop/` logs—see `malvin_tooling.md` § Repo-wide string contracts.
TRIGGER: run timing  
ADVICE: `malvin code` / `malvin kpop` / `malvin do`: `run_timing.json` + one **stdout** `TIMING:` line after the workflow body—**same `serde_json::Value`** for disk + stdout; no separate stderr “metrics hint.” Dual-failure: prefer primary workflow/ACP error—root `grounding.md` + `malvin_tooling.md` § Run timing.
TRIGGER: plan.md root vs `_malvin`  
ADVICE: Root `plan.md` vs shipped init/ACP/models (`init_cmd.rs`, tests); one-off `_malvin/**/plan.md` when cited—`malvin_tooling.md` § `malvin init` + ACP bounded retry.
TRIGGER: root plan.md informal bullets  
ADVICE: Repo-root **`./plan.md`** may hold ad-hoc requirements (not always synced with `grounding.md`). Read when cited; verify vs `src/` + tests—`malvin_debugging.md` § Root plan.
TRIGGER: KPOP MBC2, HPF, models  
ADVICE: Creative/kpop: `kpop_acp_prompt.rs`, `ops_body.inc` `run_kpop_flow_once`, `p_creative`—`malvin_tooling.md` § KPOP. **HPF** (hypothesize/predict/falsify, budget, `_malvin/.../exp_log_*.md`, incremental log, summary+tl;dr, verification-only)—`malvin_debugging.md` § KPOP + § KPOP verification. **`malvin models`** parser/ANSI: `malvin_debugging.md`. IDE search I/O → shell `rg` from repo root.
TRIGGER: Rust 2024 rand async  
ADVICE: `gen` is a keyword—use `Uniform` sampling. `Send` across `await`: `StdRng`, not `thread_rng`. Detail: `malvin_tooling.md` § Rust edition 2024.
TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` + `admin/check_untracked.sh`; `tests/init_pre_commit.rs` when `pre-commit` missing.
TRIGGER: env_path agent binary  
ADVICE: `src/env_path.rs` `agent_or_cursor_agent_bin()` — same `agent`→`cursor-agent` order as ACP spawn (`ops_body.inc`).
TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary tests use `tests/*.rs` + `env!("CARGO_BIN_EXE_malvin")` (see `init_pre_commit.rs`).
TRIGGER: ACP retry backoff  
ADVICE: `retry_policy.inc`—retriable = **timeout / deadline** substrings only; other agent/tooling errors **fail fast** (no backoff retry). `client_impl.inc`, `agent_bundle.inc` tests; **`timeout_*`** false positives—`malvin_tooling.md` § ACP bounded retry.
TRIGGER: LiteLLM token cost  
ADVICE: Prefer provider **`usage`** for billing; LiteLLM **`token_counter`** is heuristic (tiktoken/HF, fallbacks)—`malvin_tooling.md` § LiteLLM / token cost.
TRIGGER: diff thrash metric wording  
ADVICE: Byte- or path-summed edit costs and **gross/net ratios** depend on checkpoint cadence and diff math—do not treat “1.0” or low gross as proof the agent made no mistakes; state assumptions.
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
TRIGGER: clap help default punctuation  
ADVICE: In manual **`///`** on **`#[arg]`**, write **`[default: …]`** not **`(default: …)`** so help matches clap’s built-in default lines—`malvin_tooling.md` § CLI (`shared_opts.rs` pattern).
TRIGGER: llm_style layout paths  
ADVICE: **`.llm_style/malvin_tooling.md`** crate-layout + file-path ADVICEs must match **`src/`**; on renames/splits extend **`tests/cli_parity.rs`** `include_str!` guards—`malvin_tooling.md` § Tests (**`malvin_tooling path strings vs src`**).
TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; matching TRIGGER → show one TRIGGER:/ADVICE: pair. Prefer **running commands** over instruction-only when the user expects work. **Agent pacing:** distinguish product “metrics” wording from model latency/thoroughness when user-visible copy matters (`malvin_tooling.md` § Repo-wide string contracts).
