# LLM style — malvin (index)

When `.cursorrules` says so, read this file **first** on the opening message—before searches or other reads. **TRIGGER** index; detail in topic files:
- **`malvin_tooling.md`** — gates, `repo_checks`, ACP, run-timing, `malvin do`, `src/output/`, kiss, review sync, `cli_parity`, child health, `malvin code`
- **`malvin_debugging.md`** — KPOP HPF, falsify, `review_sync`, `_malvin/` plans + `ABORT:`/grounding, search fallbacks
- **`malvin_kpop_schedule.md`** — multiturn, `kpop.md`/`kpop_common` embed, prompts, `LGTM`
- **`authoring_llm_style.md`** — index <100 lines; split to topic files

## Hard constraints

TRIGGER: .kissconfig  
ADVICE: Never edit `.kissconfig`.
CONFIDENCE: 0

TRIGGER: NEVER CALL GIT  
ADVICE: Do not run git commands; users stage/commit locally. See `malvin_tooling.md` § Untracked.
CONFIDENCE: 0

TRIGGER: grounding.md never edit  
ADVICE: **Never edit `grounding.md`**. Update tests instead if `cli_parity.rs` checks fail. See `malvin_tooling.md` § Tests.
CONFIDENCE: 0

## General methodology

TRIGGER: Hypothesis vs Claim  
ADVICE: Label uncertain reasoning Hypothesis (predictions/test/confounders). Reserve Claim for cited evidence (code, logs, metrics).
CONFIDENCE: 0

TRIGGER: minimal diff  
ADVICE: Change only what the task requires; match naming, layout, and comment level; avoid drive-by refactors.
CONFIDENCE: 0

TRIGGER: review grounding  
ADVICE: Read `review.md` + `grounding.md`; update root `review.md` after fixes. See `malvin_tooling.md` § Review sync.
CONFIDENCE: 0

TRIGGER: user communication  
ADVICE: Precise prose; full paths/URLs; `startLine:endLine:path` citations; proportional length. Prefer running commands over instruction-only. See `malvin_debugging.md` § KPOP protocol completeness.
CONFIDENCE: 0

## Tooling pointers (detail in malvin_tooling.md)

TRIGGER: all checks pre-commit  
ADVICE: Full suite in `malvin_tooling.md` § Required checks. Fix every failure; rerun mid-task.
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

TRIGGER: terminal TTY word wrap  
ADVICE: `terminal_wrap.rs` handles width. See `malvin_tooling.md` § Terminal wrap.
CONFIDENCE: 0

TRIGGER: malvin do CLI  
ADVICE: Default raw vs `--cooked`. See `malvin_tooling.md` § CLI + malvin do ACP trace.
CONFIDENCE: 0

TRIGGER: Rust 2024 edition  
ADVICE: `gen` is keyword; `set_var`/`remove_var` are `unsafe`. See `malvin_tooling.md` § Rust edition 2024.
CONFIDENCE: 0

TRIGGER: ACP include layout  
ADVICE: `src/acp/` uses `include!` for kiss limits. See `malvin_tooling.md` § ACP.
CONFIDENCE: 0

TRIGGER: malvin init  
ADVICE: `src/cli/init_cmd.rs`; `default_repo/` templates. See `malvin_tooling.md` § `malvin init`.
CONFIDENCE: 0

TRIGGER: code kpop require kiss  
ADVICE: `require_kiss_for_cli_command`. See `malvin_tooling.md` § CLI kiss gate.
CONFIDENCE: 0

## KPOP pointers (detail in malvin_kpop_schedule.md / malvin_debugging.md)

TRIGGER: KPOP multiturn HPF  
ADVICE: Multiturn in `malvin_kpop_schedule.md`; HPF workflow in `malvin_debugging.md`.
CONFIDENCE: 0

TRIGGER: review_sync stale KPOP falsify  
ADVICE: `sync_review_file` / `is_lgtm` sharp edges. See `malvin_tooling.md` § Review sync; exact `LGTM` in `malvin_kpop_schedule.md`.
CONFIDENCE: 0

## Plans and grounding (detail in malvin_debugging.md)

TRIGGER: plan.md root vs _malvin  
ADVICE: Root `plan.md` vs `_malvin/**/plan.md`. See `malvin_debugging.md` § plans.
CONFIDENCE: 0

TRIGGER: grounding workflow bullets  
ADVICE: `grounding.md` workflow DSL. See `malvin_debugging.md` § plan vs grounding.
CONFIDENCE: 0

TRIGGER: search tools subagents  
ADVICE: Workspace search I/O errors → shell `rg`. See `malvin_debugging.md` § workspace search.
CONFIDENCE: 0
