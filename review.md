## Problems

1. **Resolved:** `read_exp_log_text` now returns `Result<String, String>` and surfaces I/O errors; `KpopMultiturnState::new` / `from_params` and `next_prompt` / block transitions propagate them (`src/kpop_schedule.rs`, `src/kpop_multiturn.rs`).

2. **Grounding wording (no repo edit):** The short workflow list in `grounding.md` still reads like a one-shot `header` then `kpop`. Runtime multiturn behavior (shared preamble per turn, Poisson blocks, pure MBC2 interleave) is implemented in `src/cli/kpop_flow.rs` and `src/kpop_multiturn.rs` but is not spelled out in `grounding.md`. Updating those bullets would require editing `grounding.md`, which is out of scope for this pass; see `ABORT` note in `_malvin/20260416_182803_au7les6e/result.md`.
