# Agent statuses

Short labels (1–3 words) for phases malvin commonly occupies during a run, inferred from `.malvin/logs/*/stdout.log`, `trace.jsonl`, and session artifacts. A run may hop between statuses many times; several can overlap (for example **Waiting** while **Executing**).

## Orienting

Beginning of work: reading `request.md`, recent log directories, `plan.md`, `quality_gates.log`, and often `kiss rules`. Establishes scope before substantive tool use. **HISTORY:** visible at the start of sessions such as `20260526_163201_55xw5ky3` and `20260526_173545_jrkg5otz`.

## Researching

Gathering context without changing the repo: `[Read …]`, `[Search …]`, and semantic exploration. Dominant in question-only runs (for example `20260526_170129_jm1pgecy`, which answered a `ps` question from file `o`).

## Reasoning

Thinking and planning in the agent channel: `<kpop` lines that are narrative (hypotheses, explanations, next steps) rather than tool summaries. Often appears between tool bursts and during KPop loops.

## Implementing

Changing source or tests: `[Edit …]` / write-style tool summaries. Typical when fixing bugs or adding features (for example `20260526_165108_md85hsma`, `20260526_173545_jrkg5otz`).

## Executing

Running shell commands: `[Run …]` entries (tests, probes, one-off scripts). Includes fast checks (`cargo check`) and long jobs (full `nextest`, `malvin invent`).

## Verifying

Checking that the tree meets project gates: `kiss check`, `cargo clippy`, `cargo nextest run`, sometimes batched. Malvin may run these directly (`Running \`kiss check\`` on the `malvin` channel) or the agent runs them and records output in `quality_gates.log`.

## Debugging

Investigating a failure: reading failing test output, re-running a narrowed test filter, reproducing with ad hoc commands. Distinguished from **Verifying** by starting from a known failure rather than a routine pass/fail sweep (for example `20260526_165108_md85hsma` after `hostile_agent_double_fork` failed).

## KPop cycling

Structured problem-solving: hypothesize → predict → falsify, `malvin invent`, appending `## Step K` sections to `_kpop/exp_log_*.md`, and optional `KPOP_SOLVED`. Marked in logs by `[>kpop…]` / `[kpop…]` and KPop-specific prompts.

## Waiting

Blocked on a long-running tool while malvin stays alive: heartbeats such as “May the Force be with you while you wait” or “To infinity and beyond… the next minute” on the `malvin` channel during multi-second or minute-scale `[Run …]` calls.

## Reporting

Closing out: executive summary, tl;dr, `DONE`, and `TIMING:` lines. Delivers the answer or handoff to the user and ends the visible session narrative.
