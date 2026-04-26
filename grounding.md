# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor's **Agent Client Protocol** (`agent acp`).

## Workflows

- **`malvin code`**
- If there is existing code but no .kissconfig, run `kiss clamp`.
- header; check_plan (skip with --trust-the-plan)
- header; coding_rules; implement
- header; review_1; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; review_2; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; learn (unless the run is short)
- TF TYPE I

- **`malvin sync`**
- If there is existing code but no .kissconfig, run `kiss clamp`.
- header; review_1; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; review_2; break if LGTM; concerns (check result.md for ABORT); up to max_loops times
- header; learn (unless the run is short)
- TF TYPE I

- **`malvin kpop`**
- header; coding_rules
- kpop; break if agent declares success
- mbc2 between kpop blocks (rate controlled by --p-creative)
- learn (unless the run is short)
- constraint: (kpop + mbc2) <= --max_hypotheses
- TF TYPE I

- **`malvin do`**
- do_header
- prompt
- No logging chrome to stdout. Just write the agent text.
- TF TYPE II

- **`malvin init`**
- Bootstraps a new project with pre-commit hooks and Git LFS configuration
- TF TYPE I

- **`malvin tidy`**
- If there is existing code but no .kissconfig, run `kiss clamp`.
- header; coding_rules; tidy
- header; learn (unless the run is short)
- TF TYPE I

## Other constraints
- No "documentation parity guards"
- All template keys (`{{ key }}`) in prompts must be resolved to their values. Assert "{{" does not appear in a prompt before sending it to the ACP.
- grounding.md and .kissconfig cannot be changed by agents. Before the first agent call in any workflow (after optionally calling `kiss clamp`), back up both files. After each agent call, silently restore both files from the backup.


## Text formatting (TF)
TYPE I:
- Transform from json and coalesce.
- Use word wrap.
- Format the Markdown.
- Use gray for thought text and white for regular text.
- Use colors to differentiate "from agent" tags from "to agent" tags.

TYPE II:
- Transform from json and coalesce.
- Use word wrap
- Do not format Markdown. No colors, either.
- Only include thought text in stdout if `--thoughts` is specified. Always include thought text in log-file output. Use gray for thought text and white for regular text.

