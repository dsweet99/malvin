# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor’s **Agent Client Protocol** (`agent acp`).

## Repo style file (optional)

Relative to the workflow working directory, **`.style/main.md`** may supply optional injected style for the first applicable coder/reviewer `session/prompt` turns (trimmed; whitespace-only is ignored). This is the default path used by the agent client unless overridden in code.

## Workflows

- **`malvin code`**
- header
- coding_rules
- implement
- review_1; kpop review.md; break if LGTM; concerns; up to max_loops times
- review_2; kpop review.md; break if LGTM; concerns; up to max_loops times
- learn

- **`malvin kpop`**
- header
- kpop
- learn

- **`malvin do`**
- prompt (default raw user text; `header.md` + request when `--cooked`)

## Outgoing prompts (stdout)

Each `session/prompt` prints one `[stem...]` line (timestamp-prefixed like other CLI stdout) followed by the full prompt body.

When **tee** is on, trace setup may mirror tagged lines to stdout during the same prompt. For split **`malvin do --cooked`** traces, those lines (`>style`, collapsed `>header`, then `>prompt`) are emitted **before** the `[do...]` bracket line and full body. Other prompts tee `>` lines in trace order, then the `[stem...]` line and body. Tee colors and `NO_COLOR` follow `src/output/` helpers.

For **`learn`** stems, tee does not mirror outgoing `>learn` lines to stdout (disk trace is still complete); the `[learn...]` line and full prompt body still print (see `acp_tee_echo_outgoing_prompt_lines` in `src/acp/session_trace.rs`).

## Further I/O contracts

Prefixed log layout, ACP tee direction, and related behavior: `src/output/`, `.llm_style/malvin_tooling.md` (stdout / tee).

## Run timing

- `run_timing.json` under the run directory
- One **stdout** summary line (`TIMING: ` prefix before fields)

## Grounding file (`grounding.md`)

For **`malvin code`** and **`malvin kpop`**, the workspace `grounding.md` is copied to `~/.malvin/groundings/<id>/grounding.md` at startup when present. **`malvin code`** restores that snapshot before each review attempt and again after the workflow. **`malvin kpop`** restores after its ACP body.

## Constraints
- In **`malvin code`**, **`malvin kpop`**, and **`malvin do`** (ACP-driven workflows), do not perform mutating git operations; use git read-only if you need repository state. Bootstrapping such as **`malvin init`** may invoke git tooling where that command’s docs describe (for example Git LFS setup).
