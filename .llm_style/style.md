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
CONFIDENCE: 3

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; `startLine:endLine:path` citations; proportional length. Prefer running commands over instruction-only. See `malvin_debugging.md` § KPOP protocol completeness.
CONFIDENCE: 0

## Tooling (detail in malvin_tooling.md)

TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks. Fix every failure; rerun mid-task.
CONFIDENCE: 1

TRIGGER: behavioral tests first
ADVICE: Prefer runtime behavior assertions over `include_str(...).contains(...)` guards. For `malvin do` and `malvin code`, assert exact stdout line order/content and absence of protocol/tag leakage (`"jsonrpc"`, `<do`, `:[`); avoid `--no-tee` when validating stdout behavior.
CONFIDENCE: 0

TRIGGER: kiss check and limits  
ADVICE: `kiss check .` (full project). See `malvin_tooling.md` § kiss.
CONFIDENCE: 0

TRIGGER: child health ACP silence  
ADVICE: `src/child_health/` module. See `malvin_tooling.md` § Child health.
CONFIDENCE: 0

TRIGGER: run timing  
ADVICE: `run_timing.json` + stdout `TIMING: ` line. See `malvin_tooling.md` § Run timing.
CONFIDENCE: 0

TRIGGER: stdout stderr log header  
ADVICE: Route through `src/output/mod.rs`. See `malvin_tooling.md` § Prefixed log lines.
CONFIDENCE: 0

TRIGGER: malvin do CLI  
ADVICE: Default raw vs `--cooked`; raw `do_header` uses `workflow_context_paths_only` (no `kpop_common` preload), cooked `header` uses `workflow_context` for `{{ kpop }}` in custom templates. See `malvin_tooling.md` § CLI.
CONFIDENCE: 0

TRIGGER: Rust 2024 edition  
ADVICE: `gen` is keyword; `set_var`/`remove_var` are `unsafe`. See `malvin_tooling.md` § Rust edition 2024.
CONFIDENCE: 0

TRIGGER: ACP include layout  
ADVICE: `src/acp/` uses `include!` for kiss limits. See `malvin_tooling.md` § ACP.
CONFIDENCE: 0

TRIGGER: malvin init and kiss gate  
ADVICE: `init_cmd.rs`, `default_repo/`, and `require_kiss_for_malvin` / `require_kiss_for_cli_command` (`init`, `code`, `kpop`). See `malvin_tooling.md` § `malvin init` and § CLI kiss gate.
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
