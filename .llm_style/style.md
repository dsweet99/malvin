# LLM style — malvin (index)

When `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail: `./.llm_style/malvin_tooling.md` (gates, **`repo_checks`**, ACP, run-timing merge, **malvin do**, **`src/output/`**, **kiss**, **review sync**, **`cli_parity`**, child health, **`malvin code`**), `./.llm_style/malvin_debugging.md` (KPOP HPF, falsify, **`review_sync`**, `_malvin/` **plans + `ABORT:`/grounding**, search fallbacks), `./.llm_style/malvin_kpop_schedule.md` (**multiturn**, **`kpop.md`/`kpop_common` embed**, prompts, **`LGTM`**), `./.llm_style/authoring_llm_style.md` (index **<100** lines; split to topic files).
TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks (Rust + **`pytest -sv tests`** with **`PYTHONPATH=.`** when tests import the repo); **`cargo clippy`** must match `.pre-commit-config.yaml` `entry:` verbatim. Fix every failure; no `# noqa` except for correctness; no test-cheating. Rerun mid-task (kiss limits); parallelize independent checks. **`clippy::double_must_use`:** do not add `#[must_use]` on `fn` that already returns a `#[must_use]` type (e.g. `Result`). **Pre-commit** runs ruff + clippy + kiss + `admin/check_untracked.sh` but **not** `cargo test`/`pytest`—run those manually.
TRIGGER: kiss check and limits  
ADVICE: `kiss check .` (full project); see `.kissignore`. Limits: `lines_per_file`, `calls_per_function`, `max_indentation_depth`, **duplication**, **concrete_types_per_file`. **Remove `stringify!()` hacks** when real integration tests exist. **Split monolithic tests** into focused functions. Use `.kissignore` for genuinely untestable code. See `malvin_tooling.md` § kiss.
TRIGGER: kiss structural refactors  
ADVICE: `arguments_per_function` → group parameters in a `struct` (e.g. `AcpTeeLineFmt` in `acp_tee.rs`). `calls_per_function` on a workflow entrypoint → extract a helper (e.g. `run_repo_workspace_gates` in `repo_checks.rs`). `lines_per_file` on `cli/mod.rs` → split submodules (e.g. `exit.rs`). See `malvin_tooling.md` § Kiss structural refactors.
TRIGGER: repo_checks workspace  
ADVICE: **`src/cli/repo_checks.rs`**: `run_repo_workspace_gates` (kiss clamp → kissconfig coverage warning → `pre-commit run --all-files` or warn if no `.pre-commit-config.yaml`). **`run_code`**, **`run_kpop`**, **`run_do`**. Surface `.kissconfig` read/parse errors; failed `pre-commit` includes exit code + stdout/stderr (trimmed). See `malvin_tooling.md` § Repo workspace gates.
TRIGGER: clippy doc first paragraph  
ADVICE: **`clippy::too_long_first_doc_paragraph`**: keep the opening **`///`** paragraph short; put detail in following paragraphs. **`clippy::items_after_statements`**: `use` must come before other statements in function blocks (pre-commit uses `-D warnings`).
TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.
TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. Pre-commit `admin/check_untracked.sh` fails on untracked `.rs`/`.py`—without `git add`, merge new tests into tracked `tests/*.rs` (e.g. `cli_parity.rs`); see `malvin_tooling.md` § Untracked.
TRIGGER: clap help command order  
ADVICE: `malvin --help` lists subcommands in **`src/cli/args.rs`** `Commands` enum declaration order—reorder variants to change the usage list; see `malvin_tooling.md` § CLI.
TRIGGER: cli mod sibling file  
ADVICE: `src/cli/mod.rs` `mod name;` requires `src/cli/name.rs`; ship in the same change—`malvin_tooling.md` § CLI.
TRIGGER: lib artifacts submodule file  
ADVICE: Same for **`src/artifacts/mod.rs`** (e.g. `grounding_backup.rs`); add **`tests/cli_parity.rs`** `include_str!` wiring checks when a new submodule is easy to omit from commits—`malvin_tooling.md` § Artifacts + § Tests.
TRIGGER: child health ACP silence  
ADVICE: `src/child_health/` (`linux`/`macos`/`other`); `process_absent` vs `cannot_sample`; `counters_trusted`; `rpc_wait_response` races JSON-RPC `oneshot` with `evaluate_after_acp_silence`. See `malvin_tooling.md` § Child health + ACP silence.
TRIGGER: voluntary_ctxt parse  
ADVICE: Linux `parse_status_voluntary_ctxt`: after `strip_prefix`, use **`rest.trim().parse()`**—`trim_start()` leaves trailing `\r` and breaks `u64` parse. See `malvin_tooling.md` § Child health.
TRIGGER: malvin binary crate  
ADVICE: `src/cli/` is binary-only—not the `malvin` library—so `pub(crate)` on `AgentClient` fields is not visible there; use public lib methods or keep access in lib modules. Private `src/cli/*.rs` submodules: `pub fn` not `pub(crate) fn` when `clippy::redundant_pub_crate` fires. Library `///` docs must not link `[`crate::cli::...`]`—the CLI crate is separate; use plain text (“`malvin do`”, …). See `malvin_tooling.md` § Crate layout.
TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders when useful). Reserve Claim for cited evidence (code, logs, metrics).
TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.
TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; update root `review.md` after fixes (no stale “open problems”). Verify claims vs `src/` + tests—**reviewer bullets can lag** already-correct docs/code; resolve by updating `review.md` and adding **`tests/cli_parity.rs`** guards when useful. Review sync API + paths: `malvin_tooling.md` § Review sync + `review.md`.
TRIGGER: malvin do CLI  
ADVICE: **Default (raw):** `skip_repo_style`, ACP stem `raw`. **`--cooked`:** `do_trace_split` → stems `>style` / `>header` / `>prompt` (tee collapses `header` to one stdout line); types `src/acp/outgoing_prompt_trace.rs`; one repo-style read via `coder_prompt_body_with_optional_repo_style` for both compose + trace. `do_flow.rs`, `grounding.md`; regress `tests/cli_parity.rs` + `compose_coder_prompt_tests`. Detail: `malvin_tooling.md` § CLI + malvin do ACP trace.
TRIGGER: grounding.md never edit  
ADVICE: **Never edit `grounding.md`**. When `tests/cli_parity.rs` checks fail due to grounding content, update the **tests** instead. `.llm_style/*.md` must not describe removed paths—regressions guarded in `cli_parity.rs` (`include_str!`). See `malvin_tooling.md` § Tests + docs parity.
TRIGGER: grounding workflow bullets check_plan  
ADVICE: `grounding.md` workflow DSL: `;` joins phases of one ACP turn (`header; coding_rules; implement`); separate bullets = separate turns/loops; loop bounds + break conditions inline (`up to max_loops times`, `break if LGTM`); kebab-case flags (`--max-hypotheses`); **no** algorithmic detail (Poisson/retries/credit). `default_prompts/check_plan.md` is **permissive** (`"silence on details is fine—grounding.md fills gaps"`)—terse grounding bullets license plan elaborations as non-contradictions; **name phases explicitly** in `grounding.md` to make a contract enforceable. **`malvin kpop`** target: `header; kpop (break on success); mbc2 between kpop blocks (rate by --p-creative); learn; (kpop+mbc2) <= --max-hypotheses`. `"between X blocks"` implicitly skips trailing X on early termination.
TRIGGER: stdout stderr log header  
ADVICE: Route through **`src/output/mod.rs`** (`print_stdout_line`, `print_stderr_line`, `format_line`, …). **ACP tee** ANSI + direction: **`src/output/acp_tee.rs`** (`AcpTeeDirection`, `print_stdout_acp_tee_line`)—outbound vs inbound colors; wire points **`session_trace.rs`** / **`coalesce.rs`**. **Logical** text: `YYYYMMDD.HHMMSS.mmm:[who]: …` with `[who]` padded/truncated to **`LOG_TAG_INNER_WIDTH`** Unicode scalars. **Disk** and **stderr** plain `format_line` (no ANSI). Default stdout prefix coloring: dim timestamp + cyan `who` unless ACP tee path. Document in **`grounding.md`**. Detail: `malvin_tooling.md` § Prefixed log lines.
TRIGGER: terminal TTY word wrap  
ADVICE: **`terminal_wrap.rs`**: **`terminal_columns()`** = valid **`COLUMNS`** **or** **`terminal_size::terminal_size()`** (else **80**); `line_wrap_meta` → **`stdout_line_wrap_meta`** / **`stderr_line_wrap_meta`**; **`print_stdout_line`**, **`print_stderr_line`**, **`acp_tee`**, raw tee **`trace_line_write.rs`** share width rules; repeat prefix on continuations; **disk** unwrapped. **Coalesce** cap ≠ TTY wrap—`malvin_tooling.md` § Terminal wrap (TTY).
TRIGGER: outgoing prompt stdout  
ADVICE: **All** outgoing prompts: no body tee to stdout (`acp_tee_echo_outgoing_prompt_lines` returns `false`); only `[{stem}...]` announcement via `print_outgoing_prompt_log`. Disk trace keeps full `>{stem}` lines. For `do --cooked`: announce each segment (`[style...]`, `[header...]`, `[prompt...]`). Inbound `<learn`: one `[learning...]` placeholder (`prompt_stdout_replacement`). See `malvin_tooling.md` § ACP tee.
TRIGGER: source-shape regression tests  
ADVICE: After changing ACP prompt signatures or include-body call shapes, check string-based tests like `tests/review_ops_order.rs` and docs-parity tests that `include_str!` source files—they may need updates even when runtime behavior is correct.
TRIGGER: repo-wide string contracts  
ADVICE: Renaming or banning a term: `rg` repo-wide (fragments can hide inside longer words); update `default_prompts/`, `.cursorrules`, `_kpop/` logs—see `malvin_tooling.md` § Repo-wide string contracts.
TRIGGER: run timing  
ADVICE: `malvin code` / `malvin kpop` / `malvin do`: `run_timing.json` + one **stdout** line whose payload after the timestamp starts with [`RUN_TIMING_SUMMARY_PREFIX`] (**`TIMING: `** — colon plus **one ASCII space** before the first `name = value` field)—**same `serde_json::Value`** for disk + stdout; never document bare `` `TIMING:` `` without that space (`timing_merge.rs`, `run_timing/mod.rs`, `report.rs`, `grounding.md`). No separate stderr “metrics hint.”—`malvin_tooling.md` § Run timing.
TRIGGER: primary vs secondary Result  
ADVICE: **`merge_acp_and_timing_results`** and **`prefer_primary_string_errors`** (`src/cli/timing_merge.rs`) surface ACP/workflow failures over run-timing I/O or **`grounding.md` restore** failures—`malvin_tooling.md` § Error merge.
TRIGGER: rustdoc grounding repo style  
ADVICE: **`DEFAULT_REPO_STYLE_PROMPT_REL`** (`coder_style.md`) in **`src/acp/client_impl.inc`**; user contract in **`grounding.md`** section **## Repo style file**. In **`///`** cite **`## Heading`**, not `§`—`malvin_tooling.md` § Repo style + § Docs parity.
TRIGGER: plan.md root vs `_malvin`  
ADVICE: Root `plan.md` vs shipped init/ACP/models (`init_cmd.rs`, tests); **`_malvin/**/plan.md`** when cited—**may be informal** vs **`grounding.md`**; **update** after **`src/`** (stale plan + **`review.md`**). **Unavoidable `grounding.md` conflict** → line **`ABORT:`** in that dir’s **`result.md`** (never edit **`grounding.md`**). `malvin_debugging.md` § Root plan + § **plan vs grounding ABORT** + § **_malvin plan stale**; `malvin_tooling.md` § `malvin init` + ACP bounded retry.
TRIGGER: KPOP multiturn HPF models  
ADVICE: **`malvin_kpop_schedule.md`** (`kpop_multiturn.rs`, `kpop_schedule.rs`, **`kpop.md`/`kpop_common` embed**, `run_kpop_multiturn_once`, prompts, exp-log counts). **HPF** + `_malvin/.../exp_log_*.md`—`malvin_debugging.md` § KPOP. **`malvin models`** / IDE `rg` fallback—`malvin_debugging.md`. **`review.md`** must be **only** `LGTM` for `is_lgtm_str`—see **`malvin_kpop_schedule.md`**. `malvin_tooling.md` § KPOP.
TRIGGER: Rust 2024 edition  
ADVICE: `gen` is a keyword—use `Uniform` sampling; `Send` across `await`: `StdRng`, not `thread_rng`. **`set_var`/`remove_var`** are **`unsafe`** with **`unsafe_code = deny`** → **`#[allow(unsafe_code)]`** + **`unsafe { }`** in tests; **`terminal_size`** **`Width`**: **`usize::from(w.0)`**—`malvin_tooling.md` § Rust edition 2024.
TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` templates (incl. `llm_style/style.md` with TRIGGER/ADVICE pairs); auto-commit **only on fresh repos** (no prior commits); `tests/init_pre_commit.rs` for integration tests.
TRIGGER: env_path agent binary  
ADVICE: `src/env_path.rs` `agent_or_cursor_agent_bin()` — same `agent`→`cursor-agent` order as ACP spawn (`ops_body.inc`).
TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary tests use `tests/*.rs` + `env!("CARGO_BIN_EXE_malvin")` (see `init_pre_commit.rs`).
TRIGGER: code kpop require kiss  
ADVICE: **`require_kiss_for_cli_command`** (`src/cli/mod.rs`) + **`require_kiss_for_malvin`** (`src/env_path.rs`); install **`cargo install kiss-ai`**. Regress **`tests/kiss_code_kpop_path.rs`**—see **`malvin_tooling.md` § CLI kiss gate**.
TRIGGER: ACP retry backoff  
ADVICE: `retry_policy.rs`—retriable = **timeout / deadline / failed to initialize session** substrings; other errors **fail fast**. Tests in `agent_bundle.rs`; **`timeout_*`** false positives—`malvin_tooling.md` § ACP bounded retry.
TRIGGER: DEFAULT_CLI_MODEL  
ADVICE: `src/cli/shared_opts.rs`; `models_cmd` footer `{DEFAULT_CLI_MODEL}`; `default_cli_model_is_composer_2` in `tests/cli_parity.rs`.
TRIGGER: ACP include layout  
ADVICE: Much of `src/acp/` uses `include!` for kiss limits—**all files use `.rs` extension** (not `.inc`). Included `.rs` files inherit parent `use`. When renaming, update `include!()` in source **and** `include_str!()` in `tests/cli_parity.rs`. See `malvin_tooling.md`.
TRIGGER: ACP trace, JSONL, tee  
ADVICE: Live tee: stdout reader (`trace_file_write_line`, `coalesce.rs`, `reader_inline.inc`). Test-only `strip_trace_invocation_line_for_tee` (`tee_strip_tests.inc`); **no** post-prompt file tee stub—`malvin_tooling.md` § ACP traces.
TRIGGER: ACP trace labels  
ADVICE: Directional ACP tags label the **outer prompt template filename stem** (`implement.md` → `implement`, `review_1.md` → `review_1`). For **`malvin do`**, the outer prompt is **`header.md`**, so use **`header`** even though run timing still records **Implement**.
TRIGGER: ACP tests, node  
ADVICE: Mock `agent acp` children often `#!/usr/bin/env node`; `prepend_standard_path_for_child` (`transport/command.rs`). See `malvin_tooling.md` § Tests.
TRIGGER: search tools subagents  
ADVICE: Workspace search I/O errors → shell `rg` from repo root (`malvin_debugging.md`). Merge-marker sweeps: text globs / exclude `target/`—`malvin_tooling.md` § Merge markers. ≤4 parallel subagents; skip for tiny edits. **All files green:** fix every failure—no “pre-existing” hand-waving (`malvin_tooling.md` § green tree no excuses).
TRIGGER: pub contract dead_code  
ADVICE: `pub fn` count/budget helpers must stay referenced from workflow code (`debug_assert!`, planners)—otherwise `-D dead_code` on the lib; see `malvin_tooling.md` § KPOP contracts + clippy.
TRIGGER: clippy tunable const zero  
ADVICE: `pub const` threshold **0** still needs a skippable `else` for later tuning; put `index < CONST` in one `const fn` with a **single** `#[allow(clippy::absurd_extreme_comparisons)]`—kiss `attributes_per_function` may require dropping `#[inline]`; see `malvin_tooling.md` § Clippy tunable const + kiss.
TRIGGER: CLI async timing finalize  
ADVICE: After `await` ACP work, call sync `emit_run_timing_after_acp` (`src/cli/timing_merge.rs`)—avoid async helpers taking `FnOnce(&mut AgentClient) -> Fut` (lifetime errors with `&mut` + returned `Future`). See `malvin_tooling.md` § Run timing.
TRIGGER: review_sync stale KPOP falsify  
ADVICE: **`sync_review_file`** / **`is_lgtm`** sharp edges (stale artifact **`LGTM`**, read errors → **`false`**)—`malvin_tooling.md` § Review sync; **exact `LGTM` text**—`malvin_kpop_schedule.md`. **KPOP/fs falsify:** run shell checks; **`review_sync`** is lib-private—test in-crate—`malvin_debugging.md` § KPOP falsify filesystem + § KPOP bug hunt review_sync visibility.
TRIGGER: llm_style layout paths  
ADVICE: **`.llm_style/malvin_tooling.md`** crate-layout + file-path ADVICEs must match **`src/`**; on renames/splits extend **`tests/cli_parity.rs`** `include_str!` guards—`malvin_tooling.md` § Tests (**`malvin_tooling path strings vs src`**).
TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; ```startLine:endLine:path``` citations; proportional length; matching TRIGGER → show one TRIGGER:/ADVICE: pair. Prefer **running commands** over instruction-only when the user expects work. **Named workflows** (KPOP, hypothesis budgets, stamped `_malvin/` paths): full thoroughness over skipping steps—`malvin_debugging.md` § KPOP protocol completeness. **Agent pacing:** distinguish product “metrics” wording from model latency/thoroughness when user-visible copy matters (`malvin_tooling.md` § Repo-wide string contracts).
