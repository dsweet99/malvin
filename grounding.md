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

- **`malvin kpop`**
- header
- kpop; break if agent declares success
- mbc2 between kpop blocks (rate controlled by --p-creative)
- learn (unless the run is short)
- constraint: (kpop + mbc2) <= --max_hypotheses 

- **`malvin do`**
- do_header
- prompt
- No logging chrome to stdout. Just write the agent text.

- **`malvin init`**
- Bootstraps a new project with pre-commit hooks and Git LFS configuration

- **`malvin tidy`**
- If there is existing code but no .kissconfig, run `kiss clamp`.
- header; coding_rules; tidy
- header; learn (unless the run is short)

## Other constraints
- No "documentation parity guards"
- All template keys (`{{ key }}`) in prompts must be resolved to their values. Assert "{{" does not appear in a prompt before sending it to the ACP.


## Text formatting
When logging agent output to stdout in `malvin code` or `malvin kpop`
- Format the Markdown.
- Use gray for thought text and white for regular text.
- Use colors to differentiate "from agent" tags from "to agent" tags.

When logging agent output to stdout in `malvin code` or `malvin kpop` or `malvin do`
- Transform from json and coalesce.
- Use word wrap.

When logging agent output to stdout in `malvin do`
- Only include thought text in stdout if `--thoughts` is specified. Always include thought text in log-file output.


## Testing
- Each unit test finishes in <= 10 seconds.