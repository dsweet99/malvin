# Remove mpc_block_*.md — restore pre-MPC KPop

## Problem

Malvin's KPop sessions were split into a three-phase MPC workflow (`mpc_block_a/b/c.md`): find failures, write/review plan with `DONE` gate, execute work. The user request was to remove this and run KPop like before MPC: a single Popper investigation loop via `kpop_common.md` only.

## Approach

1. **Deleted** `default_prompts/mpc_block_a.md`, `mpc_block_b.md`, `mpc_block_c.md`.
2. **Simplified prompts** — all KPop sessions now render `header.md` + `kpop_common.md` only.
3. **Collapsed multiturn** — `KpopMultiturnState::next_prompt()` returns one prompt, then `None` (no A→B→C).
4. **Removed mpc_plan / DONE** — no `_kpop/mpc_plan.md` scratch file, no `DONE` early-exit signal.
5. **Split termination logic:**
   - `code` / `tidy`: early exit when workspace quality gates pass.
   - bare `kpop` / `delight` / `explain` / `revise`: no early exit; rely on `--max-loops`.
6. **Updated tests** to assert MPC workflow strings are absent and single-prompt behavior holds.

## Key files

| Area | Files |
|------|-------|
| Prompts | `default_prompts/kpop_common.md` (unchanged), deleted `mpc_block_*.md` |
| Assembly | `src/kpop_turn_prompts.rs`, `src/kpop_multiturn_prompts.rs` |
| Session | `src/kpop_progression/multiturn.rs`, `src/kpop_engine/kpop_session.rs` |
| Exit | `src/kpop_engine/run_loop_exit.rs`, `src/cli/kpop_flow_run_loop.rs` |
| Embeds | `src/prompts/defaults.rs`, `src/prompts/default_files.rs`, `src/prompts/store.rs` |
| Docs | `default_prompts/docs/kpop.md`, `malvin.md`, `code.md`, `tidy.md` |

## Verification

```bash
cargo test
```

Contract tests:
- `embedded_defaults_exclude_mpc_blocks` — no mpc_block in `DEFAULT_PROMPTS` or `default_prompts/`
- `kpop_turn_prompts_include_kpop_common_and_exp_log` — Popper loop present, MPC strings absent
- `kpop_single_prompt_then_stop` — one `next_prompt()` then `None`
- `gate_loop_early_exit_requires_gates_for_code` — code exits on gates pass only
