# LLM style — malvin (index)

When `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail in topic files:
- **`malvin_tooling.md`** — gates, ACP, `malvin do` (raw `workflow_context_paths_only` vs cooked `workflow_context`), `DoArgs` in `do_flow.rs` (not `args.rs`), `default_repo/` vs root `admin/check_untracked`, kiss, `cli_parity`, review sync, run timing, `malvin code`
- **`malvin_debugging.md`** — KPOP HPF, falsify, `_malvin/**/plan.md` (gitignore—read by path), `ABORT:`, workspace `rg` fallback
- **`malvin_kpop_schedule.md`** — multiturn, prompts, exact `LGTM` / `is_lgtm_str`
- **`malvin_evaluations.md`** — eval harness patterns (`evaluations/`, temp repos, deterministic pass/fail oracles)
- **`authoring_llm_style.md`** — index <100 lines; topic split

## Hard constraints

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.
CONFIDENCE: 0

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. See `malvin_tooling.md` § Untracked.
CONFIDENCE: 1

TRIGGER: grounding.md never edit  
ADVICE: **Never edit `grounding.md`**. Update tests instead if `cli_parity.rs` checks fail. See `malvin_tooling.md` § Tests.
CONFIDENCE: 3

## General methodology

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders). Reserve Claim for cited evidence (code, logs, metrics).
CONFIDENCE: 1

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.
CONFIDENCE: 0

TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; update root `review.md` after fixes. See `malvin_tooling.md` § Review sync.
CONFIDENCE: 5

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; `startLine:endLine:path` citations; proportional length. Prefer running commands over instruction-only. See `malvin_debugging.md` § KPOP protocol completeness.
CONFIDENCE: 0

## Tooling (detail in malvin_tooling.md)

TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks. Fix every failure; rerun mid-task.
CONFIDENCE: 1

TRIGGER: behavioral tests first
ADVICE: Prefer runtime behavior assertions over `include_str(...).contains(...)` guards. For `malvin do` and `malvin code`, assert exact stdout line order/content and absence of protocol/tag leakage (`"jsonrpc"`, `<do`, `:[`); avoid `--no-tee` when validating stdout behavior.
CONFIDENCE: 1

TRIGGER: test mock helpers  
ADVICE: `tests/common/mod.rs` has `test_home_workspace()`, `write_mock_executable(path, js)`, and `acp_mock_js(preamble, prompt_handler)` builder. Use these; do not duplicate setup/shebang/chmod in test files.
CONFIDENCE: 1

TRIGGER: cfg unix test gating  
ADVICE: ACP mock tests and `write_mock_executable` / `PermissionsExt` are `#[cfg(unix)]`-only. Gate all imports used exclusively by unix tests with `#[cfg(unix)]`.
CONFIDENCE: 1

TRIGGER: coalesce word split  
ADVICE: `coalesce_flush_cap` splits at last word boundary (space) before 125-scalar cap via `coalesce_word_split_points`. See `malvin_tooling.md` § coalesce not TTY wrap. Regression: `reader_tests::coalesce_flush_cap_splits_at_word_boundary` + integration `do_stdout::do_wraps_wordy_long_text_at_word_boundaries`.
CONFIDENCE: 1

TRIGGER: malvin do output pipeline  
ADVICE: Agent chunks → `coalesce_append_chunk` (buffer 125) → `trace_file_write_line` → `trace_tee_stdout_line` → `print_tee_unprefixed_wrapped_line` → `wrap_words_bounded`. Coalescer is upstream of word-wrap. See `malvin_tooling.md` § Terminal wrap.
CONFIDENCE: 1

TRIGGER: kiss check and limits  
ADVICE: `kiss check .` (full project). See `malvin_tooling.md` § kiss.
CONFIDENCE: 0

TRIGGER: malvin do CLI  
ADVICE: Default raw vs `--cooked`; raw `do_header` uses `workflow_context_paths_only` (no `kpop_common` preload), cooked `header` uses `workflow_context` for `{{ kpop }}` in custom templates. See `malvin_tooling.md` § CLI.
CONFIDENCE: 2

TRIGGER: Rust 2024 edition  
ADVICE: `gen` is keyword; `set_var`/`remove_var` are `unsafe`. See `malvin_tooling.md` § Rust edition 2024.
CONFIDENCE: 0

TRIGGER: evaluation scripts  
ADVICE: Put harnesses in `evaluations/`; run in temp repos; assert stdout/stderr/exit code contracts; print `EVAL_PASS` only after all assertions pass. See `malvin_evaluations.md`.
CONFIDENCE: 0

## KPOP, review, plans

TRIGGER: KPOP HPF and review LGTM  
ADVICE: Multiturn and HPF: `malvin_kpop_schedule.md` / `malvin_debugging.md`. `is_lgtm_str` / `sync_review_file` sharp edges; root `review.md` must be **exactly** `LGTM` (trim) for automation—`malvin_kpop_schedule.md` § `review.md is_lgtm exact`.
CONFIDENCE: 3

TRIGGER: plan grounding search  
ADVICE: Root `plan.md` vs `_malvin/**/plan.md` (cited path—may be gitignored), `ABORT:`, `grounding.md` one-line workflow vs full CLI in `.llm_style`, workspace **search/glob I/O** → `rg` from repo root. See `malvin_debugging.md` § plans, § plan vs grounding, § workspace search.
CONFIDENCE: 2
