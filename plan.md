# plan.md — Multi-turn KPOP with interleaved pure-MBC2 turns

## Restated request

Change `malvin kpop` from a single-shot ACP prompt into a multi-turn loop with the following block structure:

1. **Turn 1**: agent runs `N_1` KPOP iterations (hypothesize + falsify, each).
2. **Turn 2**: agent runs **one pure MBC2** step (generate a boundary-pushing hypothesis; **no falsification** in this turn — that's what makes it "pure" and distinguishes it from today's `Mbc2ThenFalsify`).
3. **Turn 3**: agent runs `N_2` KPOP iterations.
4. **Turn 4**: one pure MBC2 step.
5. … repeat until done.

**Catch-up rule.** After each KPOP block, malvin inspects how many KPOP steps the agent actually completed (`M`). If `M < N_i`, malvin inserts an extra turn that asks for exactly `N_i − M` more KPOP iterations before moving on to the MBC2 turn.

This is the replacement for today's schedule, which is purely advisory text inside a single prompt (see `src/cli/kpop_flow.rs` and the `exp_log_synthetic_benchmark.md` evidence that a `--max-loops=1000` run stopped after 3 steps because nothing re-prompted the agent).

---

## What I can already answer from the codebase

### A. How to count `M` done per turn
**Answer: parse `_kpop/exp_log_*.md`.** The current KPOP prompt already requires the agent to write one `## Step K — KPOP …` / `## Step K — MBC2 …` section per iteration; the recent run at `_malvin/20260416_172542_pjcwsgte/_kpop/exp_log_synthetic_benchmark.md` shows this format works. We can count `^## Step \d+ — (KPOP|MBC2)` entries that appeared between the pre-turn and post-turn snapshots of that file. No new marker format needed.

### B. What "pure MBC2" means
**Answer: one MBC2 hypothesis only, no falsify.** User wrote "pure MBC2". Today's `KpopScheduleStep::Mbc2ThenFalsify` bundles an MBC2 generate + an immediate falsify; the multi-turn design separates those: MBC2 turn emits **only** a hypothesis; the next KPOP turn may pick it up and falsify it (or ignore it). This matches `default_prompts/mbc2.md` which already says "Do not evaluate or prune yet."

### C. Catch-up turn semantics
**Answer: "literal remainder" turn.** User wrote "insert another turn for `N_i − M` kpops". That's what we do — not a "continue the block" reprompt, not a fresh retry, just another block of size `N_i − M`. If the catch-up turn *also* falls short, we iterate: insert another turn for the new remainder, with a safety cap (see Q below).

### D. Inter-turn state
**Answer: the `_kpop/exp_log_*.md` file is the source of truth.** Each turn's prompt:
- includes the same KPOP/Hypothesis-vs-Claim definitions and coding rules that today's single-prompt includes,
- points the agent at the existing exp-log path,
- states "turn budget: exactly `K` KPOP steps" or "turn budget: exactly one MBC2 step",
- says "read the prior entries in `exp_log_*.md` before starting; do not repeat a falsified hypothesis".

### E. Where the code changes
Rough sketch (subject to your answers below):
- `src/cli/kpop_flow.rs`: replace the single `kpop_run_acp` call with a loop driver.
- `src/kpop_schedule.rs`: delete randomized `Mbc2ThenFalsify` placement; replace with a deterministic block-sequence planner (`plan_kpop_blocks(total_budget, block_size) -> Vec<Turn>`).
- `default_prompts/kpop.md` and `default_prompts/mbc2.md`: add per-turn wrappers that fix the turn budget (`"You are in a KPOP turn. Complete exactly K iterations and then stop."` / `"You are in a pure-MBC2 turn. Produce exactly one MBC2 hypothesis. Do not falsify."`).
- `src/acp/ops_body.rs` (`run_kpop_flow_once`): extend or wrap to support multiple sequential prompt dispatches (depends on Q5 below).
- Tests: `src/kpop_schedule.rs` unit tests for the block planner; `tests/cli_parity.rs` tweaks; a new integration test with a stub agent that completes partial turns, to exercise the catch-up path.

---

## Questions for you (multiple choice)

### Q1 — How is `N_i` sized?
- **(a)** Constant `N` for every KPOP block; configurable via a new flag like `--kpop-block-size=10` (default: **10**). Simplest.
- **(b)** Escalating: `N_1 = 5, N_2 = 10, N_3 = 20, N_k = min(cap, 5·2^(k−1))`. Encourages early MBC2 diversification, later exploitation.
- **(c)** Randomized around a mean (e.g. Poisson with mean 10).
- **(d)** Use today's `--p-creative` to pick block boundaries inline, one-block-at-a-time (essentially the current schedule, just binding).

### Q2 — What is the global stop condition ("until done")?
- **(a)** Keep `--max-loops=N` as a hard cap on the **total KPOP iterations** across all blocks. Stop when cumulative KPOP count reaches `N` **or** when the agent explicitly says it has solved the problem (one agreed marker string in its final turn output).
- **(b)** `--max-loops` on **total turns** (KPOP-blocks + MBC2 turns together), agent-declared success otherwise.
- **(c)** Run until a user-supplied external check passes (new `--success-cmd "cargo test && pytest -sv tests"`); `--max-loops` is only a safety cap.
- **(d)** Combination: `(a)` plus the external check from `(c)` when `--success-cmd` is provided.

### Q3 — ACP session lifecycle across turns
- **(a)** **One long ACP session**; each turn is a new `prompt` request in the same session. Preserves context (agent remembers prior hypotheses implicitly) but grows token usage roughly linearly in turns.
- **(b)** **New ACP session per turn**. Zero cross-turn context except what's written to `_kpop/exp_log_*.md`; predictable token cost per turn; forces the agent to re-read the exp log each time.
- **(c)** **One session per `[KPOPs, MBC2]` pair**, new session after each pair. Middle ground.

### Q4 — Overshoot handling (agent runs more than `N_i` in a KPOP block)
- **(a)** Accept the overshoot and credit it toward the **next** KPOP block's budget. Cheap, natural.
- **(b)** Accept the overshoot but don't credit it (next block still asks for `N_{i+1}`). Keeps block sizes honest.
- **(c)** Treat overshoot as a hard error (fail fast on the turn).

### Q5 — Pure-MBC2 turn shortfall
If the pure-MBC2 turn produces zero MBC2 hypotheses:
- **(a)** One automatic retry turn, then proceed to the next KPOP block regardless. Bounded.
- **(b)** Retry indefinitely until at least one hypothesis appears (with some sanity cap like 3).
- **(c)** Don't retry. Log and continue to the next KPOP block.

### Q6 — Catch-up-turn safety cap
If a KPOP catch-up turn itself falls short:
- **(a)** Keep inserting catch-up turns until the block is satisfied OR `--max-loops` is exhausted. Simple; can diverge if the agent is consistently short.
- **(b)** Allow **at most** `K_catchup` catch-up attempts per block (e.g. `K_catchup = 3`). If still short, accept what we have and move on to MBC2.
- **(c)** Allow at most `K_catchup`, and if still short, **abort** the whole run with a non-zero exit.

### Q7 — CLI surface / backward compatibility
- **(a)** Replace today's single-shot KPOP wholesale; drop `KpopScheduleStep::Mbc2ThenFalsify` and the `p_creative` flag.
- **(b)** Gate new behavior behind an opt-in flag (e.g. `--multiturn`, default off); keep single-shot as the default until we're confident.
- **(c)** Opt-out flag (e.g. `--single-turn`), multiturn as the new default.

### Q8 — MBC2 turn placement
- **(a)** Exactly as user described: one pure MBC2 between every KPOP block, forever.
- **(b)** Same, but **skip** the MBC2 turn if the last KPOP block already solved the problem (agent-declared). Saves one LLM turn at the end.
- **(c)** Same as (a), but also insert a pure MBC2 turn **before** the first KPOP block ("seed the exploration").

---

## Decisions (user-selected 2026-04-16)

| Q | Choice | Meaning |
|---|---|---|
| Q1 | (c) | Block size `N_i` drawn from `Poisson(mean)` per block, where `mean = max(1, (1 − p_creative) / p_creative)`. `p_creative` is kept, repurposed as the Poisson-mean driver. |
| Q2 | (b+rename) | Budget flag renamed `--max-loops` → `--max-hypotheses`. Counts **total hypotheses emitted** (KPOP steps + MBC2 steps). Early exit on agent-declared success. |
| Q3 | (a) | **One long ACP session** across all turns; each turn = new `prompt` request inside that one session. |
| Q4 | (a) | Overshoot in a KPOP block **credits the next block's budget**. |
| Q5 | (a) | Empty pure-MBC2 turn → one automatic retry, then proceed regardless. |
| Q6 | (c) | Up to `K_catchup = 3` catch-up turns per KPOP block; if still short, **abort the whole run with non-zero exit**. |
| Q7 | (a, adjusted) | Replace single-shot wholesale. Remove `KpopScheduleStep::Mbc2ThenFalsify` and the pre-rolled `generate_kpop_schedule`. **Keep** `--p-creative` — repurposed as the Poisson-mean driver (see Q1). |
| Q8 | (b) | Strict KPOP-then-MBC2 interleave, **but skip the trailing MBC2** if the previous KPOP block emitted a success marker. |

### `p_creative` → Poisson mapping

`mean_block_size(p) = max(1, (1 − p) / p)`

| `p_creative` | Poisson mean | Intuition |
|---|---|---|
| `0.03` | ≈ 32 | long KPOP runs, rare MBC2 |
| `0.10` (default) | ≈ 9 | matches today's ratio |
| `0.5`  | 1 | MBC2 after almost every KPOP |
| `1.0`  | 1 (clamped) | MBC2 after every single KPOP |
| ≤ 0 or non-finite | MBC2 **disabled**; block mean falls back to constant 10 so catch-up / retry machinery still runs | pure-KPOP multiturn |

This reuses the existing `kpop_creative_enabled(p)` guard in `src/kpop_acp_prompt.rs` to decide whether MBC2 turns are interleaved at all.

### Implementation sketch after these choices

**Top-level loop (new).** Outline in `src/cli/kpop_flow.rs`:

```text
open one ACP session
exp_log_path = <run_dir>/_kpop/exp_log_<slug>.md
block_mean = if kpop_creative_enabled(p_creative)
              then max(1.0, (1.0 - p_creative) / p_creative)
              else 10.0
credit = 0     # overshoot carried from previous KPOP block
while hypotheses_emitted(exp_log_path) < max_hypotheses:
    N = max(1, credit + Poisson(block_mean))
    credit = 0
    attempts = 0
    kpop_before = count_kpop_entries(exp_log_path)
    while attempts <= K_catchup:
        want = N - (count_kpop_entries(exp_log_path) - kpop_before)
        if want <= 0: break
        if hypotheses_emitted(exp_log_path) >= max_hypotheses: goto DONE
        send_kpop_block_prompt(want=want, budget_left=remaining())
        attempts += 1
        if agent_declared_success(exp_log_path): goto DONE
    actual = count_kpop_entries(exp_log_path) - kpop_before
    if actual < N and attempts > K_catchup:
        return Err("KPOP block short after 3 catch-ups")
    credit = max(0, actual - N)                              # Q4
    if hypotheses_emitted(exp_log_path) >= max_hypotheses: goto DONE
    if not kpop_creative_enabled(p_creative): continue       # pure-KPOP mode: no MBC2 turn
    if agent_declared_success(exp_log_path): goto DONE       # Q8: skip trailing MBC2 on success
    mbc2_before = count_mbc2_entries(exp_log_path)
    send_pure_mbc2_prompt()
    if count_mbc2_entries(exp_log_path) == mbc2_before \
       and hypotheses_emitted(exp_log_path) < max_hypotheses:
        send_pure_mbc2_prompt()                              # Q5: one retry
DONE:
close ACP session
```

`hypotheses_emitted = count_kpop_entries + count_mbc2_entries`. Counts come from `^## Step \d+ — KPOP` and `^## Step \d+ — MBC2` line-scans of `exp_log_*.md` (the agent already emits these today, per `_malvin/20260416_172542_pjcwsgte/_kpop/exp_log_synthetic_benchmark.md`).

**Code surfaces that change:**
- `src/kpop_schedule.rs` — remove `KpopScheduleStep::Mbc2ThenFalsify`, `generate_kpop_schedule`, `schedule_requires_mbc2`, `build_scheduled_kpop_prompt` (the pre-rolled schedule is gone). Replace with: `poisson_block_size(rng, mean) -> usize` (hand-rolled Knuth), `block_mean_from_p_creative(p) -> f64`, `count_kpop_entries(path) -> usize`, `count_mbc2_entries(path) -> usize`, `agent_declared_success(path) -> bool`.
- `src/cli/kpop_flow.rs` — replace `kpop_run_acp` with the loop above.
- `src/cli/args.rs` — `KpopArgs::max_loops` renamed to `max_hypotheses`; CLI flag renamed `--max-loops` → `--max-hypotheses`. `CodeArgs::max_loops` is left alone (different concept). `p_creative` kept, help text updated to describe its new Poisson-mean role.
- `src/acp/ops_body.rs::run_kpop_flow_once` — extend (or add sibling `run_kpop_flow_multi`) so we can dispatch *N* prompts in one ACP session instead of the current 1–2.
- `default_prompts/kpop.md` — split into three per-turn templates:
  - `default_prompts/kpop_block.md` — "You are in a KPOP turn. Run exactly `{{ want }}` HPF iterations. Record each as `## Step K — KPOP …` in `{{ exp_log }}` and stop."
  - `default_prompts/mbc2_pure.md` — "You are in a pure-MBC2 turn. Produce exactly one MBC2 hypothesis. Do **not** falsify it. Record as `## Step K — MBC2 …` in `{{ exp_log }}` and stop."
  - Shared preamble (Hypothesis-vs-Claim, coding rules) factored into `default_prompts/kpop_common.md`.
- Tests: replace `src/kpop_schedule.rs` schedule tests with block-planner tests (Poisson mean mapping, block-size floor, `kpop_creative_enabled` disable path); add a new `tests/kpop_multiturn.rs` integration test with a stub agent that produces short / overshoot / empty turns to exercise the catch-up, credit, MBC2-retry, and success-early-exit paths.

### Derivative details — my proposed defaults (raise a flag if any is wrong)

1. **Success-marker convention.** Agent declares "done" by appending a final section `## KPOP_SOLVED\n<one-paragraph why>` to the exp-log. Malvin greps `^## KPOP_SOLVED\b` after each turn. Clear, greppable, mirrors existing `## Step K — …` style.
2. **Poisson implementation.** Hand-roll Knuth's algorithm (~6 lines) against `StdRng` rather than add a new `rand_distr` dependency. Floor at 1 so we never dispatch a zero-sized KPOP block.
3. **Default `--max-hypotheses`.** Keep the current `--max-loops` default of **10** so `malvin kpop @req.md` with no flags still feels familiar. (Old semantic: 10 pre-rolled slots. New semantic: 10 total emitted hypotheses. Same order of magnitude.)
4. **`p_creative` retention.** Kept, default **0.10** (unchanged). Help text rewritten: "Probability of MBC2 interleave; higher = more frequent creative-break turns and smaller KPOP blocks. 0 or non-finite disables MBC2 turns."
5. **Single long ACP session (Q3).** Reuse the existing `AgentClient` session; loop in the driver sends `prompt` requests sequentially. No new session primitives needed.
6. **Kiss / clippy.** Keep `src/kpop_schedule.rs` and `src/cli/kpop_flow.rs` under the 250-line cap by pushing the loop body into a small `KpopLoopState` struct with `next_block_size`, `run_kpop_block`, `run_mbc2_turn` methods.

---

Please confirm the five defaults above (or correct any), then I'll implement.
